use std::time::Duration;

/// Advanced configuration options for HTTP/2 behavior.
///
/// All fields are optional; if a field is `None`, Hyper's internal default for
/// that setting is used. Most applications can rely on the defaults and only
/// override `max_concurrent_streams` or timeouts for specific workloads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Http2Config {
    /// If `true`, only HTTP/2 connections are accepted on this listener.
    /// If `false`, HTTP/1.1 and HTTP/2 are both supported (negotiated by Hyper).
    pub http2_only: bool,
    /// Maximum number of concurrent streams allowed per HTTP/2 connection.
    pub max_concurrent_streams: Option<u32>,
    /// Initial stream-level flow control window size.
    pub initial_stream_window_size: Option<u32>,
    /// Initial connection-level flow control window size.
    pub initial_connection_window_size: Option<u32>,
    /// Enable or disable Hyper's adaptive flow control window behavior.
    pub adaptive_window: Option<bool>,
    /// Maximum allowed HTTP/2 frame size in bytes.
    pub max_frame_size: Option<u32>,
    /// Maximum size of the header list (in octets) that is allowed.
    pub max_header_list_size: Option<u32>,
    /// Interval at which HTTP/2 PING frames are sent to keep the connection alive.
    pub keep_alive_interval: Option<Duration>,
    /// Timeout waiting for a PING ACK before considering the connection dead.
    pub keep_alive_timeout: Option<Duration>,
    /// Whether to send keep-alive PINGs even when the connection is idle.
    pub keep_alive_while_idle: Option<bool>,
}
