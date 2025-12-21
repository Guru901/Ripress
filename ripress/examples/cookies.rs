//! Cookies Example
//!
//! Demonstrates cookie management in Ripress:
//! - Setting cookies with various options
//! - Reading cookies from requests
//! - Clearing cookies
//! - Secure cookie configurations

use ripress::{
    app::App,
    req::HttpRequest,
    res::{
        response_cookie::{CookieOptions, CookieSameSiteOptions},
        HttpResponse,
    },
    types::RouterFns,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Set a simple cookie
    app.get(
        "/set-cookie",
        |_: HttpRequest, res: HttpResponse| async move {
            res.set_cookie("simple_cookie", "hello_world", None)
                .json(json!({
                    "message": "Cookie set!",
                    "cookie_name": "simple_cookie"
                }))
        },
    );

    // Set a secure session cookie
    app.get("/login", |_: HttpRequest, res: HttpResponse| async move {
        let options = CookieOptions {
            http_only: true,
            secure: true,
            same_site: CookieSameSiteOptions::Strict,
            path: Some("/"),
            max_age: Some(3600), // 1 hour
            ..Default::default()
        };

        res.set_cookie("session_token", "abc123xyz", Some(options))
            .status(200)
            .json(json!({
                "message": "Logged in successfully",
                "expires_in": "1 hour"
            }))
    });

    // Set multiple cookies
    app.get(
        "/set-multiple",
        |_: HttpRequest, res: HttpResponse| async move {
            res.set_cookie("user_id", "12345", None)
                .set_cookie(
                    "theme",
                    "dark",
                    Some(CookieOptions {
                        path: Some("/"),
                        max_age: Some(2592000), // 30 days
                        ..Default::default()
                    }),
                )
                .set_cookie("language", "en", None)
                .json(json!({
                    "message": "Multiple cookies set"
                }))
        },
    );

    // Read cookies from request
    app.get(
        "/check-cookies",
        |req: HttpRequest, res: HttpResponse| async move {
            let cookie_header = req.headers.get("cookie").unwrap_or("No cookies found");

            res.json(json!({
                "cookies": cookie_header,
                "message": "Check your cookies"
            }))
        },
    );

    // Clear a cookie (logout)
    app.get("/logout", |_: HttpRequest, res: HttpResponse| async move {
        res.clear_cookie("session_token").json(json!({
            "message": "Logged out successfully"
        }))
    });

    // Set cookie with all options
    app.get(
        "/advanced-cookie",
        |_: HttpRequest, res: HttpResponse| async move {
            let options = CookieOptions {
                http_only: true,
                secure: true,
                same_site: CookieSameSiteOptions::Lax,
                path: Some("/api"),
                domain: Some("localhost"),
                max_age: Some(7200),
                expires: None,
            };

            res.set_cookie("api_token", "secure_token_xyz", Some(options))
                .json(json!({
                    "message": "Advanced cookie set with all options",
                    "config": {
                        "http_only": true,
                        "secure": true,
                        "same_site": "Lax",
                        "path": "/api",
                        "max_age": 7200
                    }
                }))
        },
    );

    // Remember me functionality
    app.post(
        "/remember-me",
        |_: HttpRequest, res: HttpResponse| async move {
            let options = CookieOptions {
                http_only: true,
                secure: true,
                same_site: CookieSameSiteOptions::Strict,
                path: Some("/"),
                max_age: Some(2592000), // 30 days
                ..Default::default()
            };

            res.set_cookie("remember_token", "long_lived_token", Some(options))
                .json(json!({
                    "message": "Remember me enabled",
                    "duration": "30 days"
                }))
        },
    );

    // Clear all cookies
    app.get(
        "/clear-all",
        |_: HttpRequest, res: HttpResponse| async move {
            res.clear_cookie("simple_cookie")
                .clear_cookie("session_token")
                .clear_cookie("user_id")
                .clear_cookie("theme")
                .clear_cookie("language")
                .clear_cookie("remember_token")
                .json(json!({
                    "message": "All cookies cleared"
                }))
        },
    );

    println!("🍪 Cookies example server starting on http://127.0.0.1:3000");
    println!("\nTry these endpoints:\n");
    println!("Set simple cookie:");
    println!("  curl http://127.0.0.1:3000/set-cookie -v\n");
    println!("Login (secure cookie):");
    println!("  curl http://127.0.0.1:3000/login -v\n");
    println!("Set multiple cookies:");
    println!("  curl http://127.0.0.1:3000/set-multiple -v\n");
    println!("Check cookies:");
    println!("  curl http://127.0.0.1:3000/check-cookies -H 'Cookie: session_token=abc123xyz'\n");
    println!("Logout:");
    println!("  curl http://127.0.0.1:3000/logout -v\n");

    app.listen(3000, || {}).await;
    Ok(())
}
