#[cfg(test)]
mod tests {
    use actix_web::web::Bytes;

    use crate::{app::App, websocket::WebSocket};

    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    #[test]
    fn test_websocket_new() {
        let ws = WebSocket::new("/test");
        assert_eq!(ws.path, "/test");
    }

    #[test]
    fn test_websocket_callbacks() {
        let mut ws = WebSocket::new("/test");

        // Test text message callback
        let test_msg = String::from("test message");
        let msg_clone = test_msg.clone();

        ws.on_text(move |msg| {
            assert_eq!(msg, msg_clone);
        });

        // Test binary callback
        let test_bytes = Bytes::from_static(b"test binary");
        let bytes_clone = test_bytes.clone();

        ws.on_binary(move |data| {
            assert_eq!(data, bytes_clone);
        });

        // Test connect callback
        let connect_called = Arc::new(Mutex::new(false));
        let connect_called_clone = connect_called.clone();

        ws.on_connect(move |_, _| {
            let mut called = connect_called_clone.lock().unwrap();
            *called = true;
        });

        // Test disconnect callback
        let disconnect_called = Arc::new(Mutex::new(false));
        let disconnect_called_clone = disconnect_called.clone();
        ws.on_disconnect(move |_, _| {
            let mut disconnect_called = disconnect_called_clone.lock().unwrap();
            *disconnect_called = true;
        });
    }

    #[test]
    fn test_websocket_registration() {
        let mut app = App::new();
        let ws = WebSocket::new("/ws");
        ws.register(&mut app);

        assert!(app.has_ws);
        assert_eq!(app.ws_path, "/ws");
    }

    #[test]
    fn test_websocket_heartbeat() {
        let ws = WebSocket::new("/test");
        let initial_hb = ws.hb;
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(ws.hb, initial_hb); // Heartbeat shouldn't change without explicit update
    }
}
