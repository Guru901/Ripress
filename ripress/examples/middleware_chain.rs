//! Middleware Chain Example
//!
//! Demonstrates middleware composition in Ripress:
//! - Request logging
//! - Authentication
//! - CORS handling
//! - Request timing
//! - Multiple middleware chains

use ripress::{app::App, req::HttpRequest, res::HttpResponse, types::RouterFns};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Global middlewares (apply to all routes)
    app.use_pre_middleware(None, |mut req, _| async move {
        use std::time::{SystemTime, UNIX_EPOCH};
        let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let timestamp = since_epoch.as_nanos();
        // Optionally, add pointer address for more randomness
        let ptr_id = format!("{:p}", &req);

        let request_id = format!("REQ-{}-{}", timestamp, ptr_id);

        // Store request ID in headers for downstream use
        req.headers.insert(
            hyper::header::HeaderName::from_static("x-request-id"),
            &request_id,
        );

        println!("🔖 Request ID: {}", request_id);

        (req, None)
    });
    app.use_post_middleware(None, |req, res| async move {
        let method = req.method.clone();
        let path = req.origin_url.to_string();

        println!("📝 {} {}", method, path);
        println!("   ↳ Status: {}", res.status_code());
        (req, Some(res))
    });

    // Public routes (no auth required)
    app.get("/", |_: HttpRequest, res: HttpResponse| async move {
        res.json(json!({
            "message": "Welcome to Middleware Chain Example",
            "endpoints": {
                "public": ["/", "/health", "/public"],
                "protected": ["/api/users", "/api/admin"]
            }
        }))
    });

    app.get("/health", |_: HttpRequest, res: HttpResponse| async move {
        // Generate a basic ISO-8601-like timestamp using only std.
        let now = std::time::SystemTime::now();
        let timestamp_str = match now.duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => {
                let secs = dur.as_secs() as i64;
                let nsecs = dur.subsec_nanos();
                let tm = secs;
                format!("{}.{:09}Z", tm, nsecs)
            }
            Err(_) => "unknown".to_string(),
        };
        res.json(json!({
            "status": "healthy",
            "timestamp": timestamp_str
        }))
    });

    app.get("/public", |_: HttpRequest, res: HttpResponse| async move {
        res.json(json!({
            "message": "This is a public endpoint",
            "auth_required": false
        }))
    });

    app.use_pre_middleware("/api", |req, _| async move {
        let auth_header = req.headers.get("authorization");

        match auth_header {
            Some(_) => {
                println!("✅ Authentication successful");
                return (req, None);
            }
            None => {
                println!("❌ No token provided");
                let res = HttpResponse::new().status(401).json(json!({
                    "error": "Authentication required"
                }));

                return (req, Some(res));
            }
        }
    });

    app.get(
        "/api/users",
        |_: HttpRequest, res: HttpResponse| async move {
            res.json(json!({
                "users": [
                    {"id": 1, "name": "Alice"},
                    {"id": 2, "name": "Bob"}
                ]
            }))
        },
    );

    app.get(
        "/api/admin",
        |_: HttpRequest, res: HttpResponse| async move {
            res.json(json!({
                "message": "Welcome to admin panel",
                "permissions": ["read", "write", "delete"]
            }))
        },
    );

    // Rate-limited endpoint

    app.get(
        "/api/limited",
        |_: HttpRequest, res: HttpResponse| async move {
            res.json(json!({
                "message": "This endpoint is rate-limited",
                "limit": "5 requests per minute"
            }))
        },
    );

    // Multiple middleware chain
    app.post(
        "/api/sensitive",
        |_: HttpRequest, res: HttpResponse| async move {
            res.json(json!({
                "message": "Sensitive operation completed",
                "success": true
            }))
        },
    );

    println!("🔗 Middleware Chain example server starting on http://127.0.0.1:3000");
    println!("\nMiddleware stack:");
    println!("  1️⃣ Request ID (global)");
    println!("  2️⃣ Logger (global)");
    println!("  3️⃣ Timing (global)");
    println!("  4️⃣ CORS (global)");
    println!("  5️⃣ Auth (protected routes)");
    println!("  6️⃣ Rate Limit (specific routes)");
    println!("\nTry these curl commands:\n");
    println!("# Public endpoint:");
    println!("curl http://127.0.0.1:3000/public\n");
    println!("# Protected endpoint (no auth - will fail):");
    println!("curl http://127.0.0.1:3000/api/users\n");
    println!("# Protected endpoint (with auth):");
    println!("curl http://127.0.0.1:3000/api/users -H 'Authorization: Bearer secret_token_123'\n");
    println!("# Rate-limited endpoint (try multiple times):");
    println!("for i in {{1..7}}; do curl http://127.0.0.1:3000/api/limited; echo; done\n");

    app.listen(3000, || {}).await;
    Ok(())
}
