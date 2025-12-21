//! Router Versioning Example
//!
//! Demonstrates API versioning strategies:
//! - URL-based versioning (/v1/, /v2/)
//! - Header-based versioning
//! - Multiple API versions coexisting
//! - Version deprecation handling

use ripress::{
    app::App,
    req::{body::json_data::JsonBody, route_params::Params, HttpRequest},
    res::HttpResponse,
    types::RouterFns,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

// V1 Models
#[derive(Debug, Serialize, Deserialize)]
struct UserV1 {
    id: u32,
    name: String,
    email: String,
}

impl ripress::req::body::json_data::FromJson for UserV1 {
    fn from_json(data: &ripress::req::body::RequestBodyContent) -> Result<Self, String> {
        if let ripress::req::body::RequestBodyContent::JSON(json_val) = data {
            serde_json::from_value(json_val.clone()).map_err(|e| e.to_string())
        } else {
            Err("Expected JSON body".to_string())
        }
    }
}

// V2 Models (enhanced with more fields)
#[derive(Debug, Serialize, Deserialize)]
struct UserV2 {
    id: u32,
    first_name: String,
    last_name: String,
    email: String,
    phone: Option<String>,
    created_at: String,
}

impl ripress::req::body::json_data::FromJson for UserV2 {
    fn from_json(data: &ripress::req::body::RequestBodyContent) -> Result<Self, String> {
        if let ripress::req::body::RequestBodyContent::JSON(json_val) = data {
            serde_json::from_value(json_val.clone()).map_err(|e| e.to_string())
        } else {
            Err("Expected JSON body".to_string())
        }
    }
}

// V3 Models (complete redesign)
#[derive(Debug, Serialize, Deserialize)]
struct UserV3 {
    uuid: String,
    profile: UserProfile,
    settings: UserSettings,
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserProfile {
    full_name: String,
    email: String,
    phone: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserSettings {
    notifications_enabled: bool,
    theme: String,
    language: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    created_at: String,
    updated_at: String,
    last_login: Option<String>,
}

impl ripress::req::body::json_data::FromJson for UserV3 {
    fn from_json(data: &ripress::req::body::RequestBodyContent) -> Result<Self, String> {
        if let ripress::req::body::RequestBodyContent::JSON(json_val) = data {
            serde_json::from_value(json_val.clone()).map_err(|e| e.to_string())
        } else {
            Err("Expected JSON body".to_string())
        }
    }
}

#[derive(Debug)]
struct UserId {
    id: u32,
}

impl ripress::req::route_params::FromParams for UserId {
    fn from_params(params: &ripress::req::route_params::RouteParams) -> Result<Self, String> {
        let id = params
            .get("id")
            .ok_or("Missing id parameter")?
            .parse()
            .map_err(|_| "Invalid id format")?;
        Ok(UserId { id })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Root endpoint
    app.get("/", |_: HttpRequest, res: HttpResponse| async move {
        res.json(json!({
            "service": "Ripress API",
            "versions": {
                "v1": {
                    "status": "deprecated",
                    "sunset_date": "2025-12-31",
                    "base_url": "/api/v1"
                },
                "v2": {
                    "status": "stable",
                    "base_url": "/api/v2"
                },
                "v3": {
                    "status": "beta",
                    "base_url": "/api/v3"
                }
            },
            "documentation": "https://docs.example.com/api"
        }))
    });

    // ==================== API V1 (Deprecated) ====================

    app.get(
        "/api/v1/users",
        |_: HttpRequest, res: HttpResponse| async move {
            res.set_header("X-API-Deprecated", "true")
                .set_header("X-API-Sunset", "2025-12-31")
                .set_header("X-API-Migration", "https://docs.example.com/migration-v2")
                .json(json!({
                    "warning": "This API version is deprecated. Please migrate to v2",
                    "data": [
                        UserV1 {
                            id: 1,
                            name: String::from("Alice Johnson"),
                            email: String::from("alice@example.com")
                        },
                        UserV1 {
                            id: 2,
                            name: String::from("Bob Smith"),
                            email: String::from("bob@example.com")
                        }
                    ]
                }))
        },
    );

    app.get(
        "/api/v1/users/:id",
        |params: Params<UserId>, res: HttpResponse| async move {
            res.set_header("X-API-Deprecated", "true").json(json!({
                "warning": "This API version is deprecated",
                "data": UserV1 {
                    id: params.id,
                    name: String::from("Alice Johnson"),
                    email: String::from("alice@example.com")
                }
            }))
        },
    );

    app.post(
        "/api/v1/users",
        |body: JsonBody<UserV1>, res: HttpResponse| async move {
            res.set_header("X-API-Deprecated", "true")
                .status(201)
                .json(json!({
                    "warning": "This API version is deprecated",
                    "data": {
                        "id": 3,
                        "name": body.name,
                        "email": body.email
                    }
                }))
        },
    );

    // ==================== API V2 (Stable) ====================

    app.get(
        "/api/v2/users",
        |_: HttpRequest, res: HttpResponse| async move {
            res.set_header("X-API-Version", "2.0").json(json!({
                "data": [
                    UserV2 {
                        id: 1,
                        first_name: "Alice".to_string(),
                        last_name: "Johnson".to_string(),
                        email: "alice@example.com".to_string(),
                        phone: Some("+1234567890".to_string()),
                        created_at: "2024-01-15T10:30:00Z".to_string()
                    },
                    UserV2 {
                        id: 2,
                        first_name: "Bob".to_string(),
                        last_name: "Smith".to_string(),
                        email: "bob@example.com".to_string(),
                        phone: None,
                        created_at: "2024-02-20T14:45:00Z".to_string()
                    }
                ],
                "pagination": {
                    "page": 1,
                    "per_page": 10,
                    "total": 2
                }
            }))
        },
    );

    app.get(
        "/api/v2/users/:id",
        |params: Params<UserId>, res: HttpResponse| async move {
            res.set_header("X-API-Version", "2.0").json(json!({
                "data": UserV2 {
                    id: params.id,
                    first_name: "Alice".to_string(),
                    last_name: "Johnson".to_string(),
                    email: "alice@example.com".to_string(),
                    phone: Some("+1234567890".to_string()),
                    created_at: "2024-01-15T10:30:00Z".to_string()
                }
            }))
        },
    );

    app.post(
        "/api/v2/users",
        |body: JsonBody<UserV2>, res: HttpResponse| async move {
            let now = std::time::SystemTime::now();
            let created_at = match now.duration_since(std::time::UNIX_EPOCH) {
                Ok(duration) => {
                    let secs = duration.as_secs();
                    let nsecs = duration.subsec_nanos();
                    // Format as "seconds.nanosecondsZ" (approximate, not true RFC3339)
                    format!("{}.{}Z", secs, nsecs)
                }
                Err(_) => "unknown".to_string(),
            };

            let first_name = &body.first_name;
            let last_name = &body.last_name;
            let email = &body.email;
            let phone = &body.phone;

            res.set_header("X-API-Version", "2.0")
                .status(201)
                .json(json!({
                    "data": UserV2 {
                        id: 3,
                        first_name: first_name.to_string(),
                        last_name: last_name.to_string(),
                        email: email.to_string(),
                        phone: phone.clone(),
                        created_at: created_at.to_string(),
                    }
                }))
        },
    );

    app.put(
        "/api/v2/users/:id",
        |(params, body): (Params<UserId>, JsonBody<UserV2>), res: HttpResponse| async move {
            let first_name = &body.first_name;
            let last_name = &body.last_name;
            let email = &body.email;
            let created_at = &body.created_at;
            let phone = &body.phone;

            res.set_header("X-API-Version", "2.0").json(json!({
                "data": UserV2 {
                    id: params.id,
                    first_name: first_name.to_string(),
                    last_name: last_name.to_string(),
                    email: email.to_string(),
                    phone: phone.clone(),
                    created_at: created_at.to_string(),
                }
            }))
        },
    );

    // ==================== API V3 (Beta) ====================

    app.get(
        "/api/v3/users",
        |_: HttpRequest, res: HttpResponse| async move {
            res.set_header("X-API-Version", "3.0-beta")
            .set_header("X-API-Status", "beta")
            .json(json!({
                "data": [
                    UserV3 {
                        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                        profile: UserProfile {
                            full_name: "Alice Johnson".to_string(),
                            email: "alice@example.com".to_string(),
                            phone: Some("+1234567890".to_string()),
                            avatar_url: Some("https://example.com/avatars/alice.jpg".to_string())
                        },
                        settings: UserSettings {
                            notifications_enabled: true,
                            theme: "dark".to_string(),
                            language: "en".to_string()
                        },
                        metadata: Metadata {
                            created_at: "2024-01-15T10:30:00Z".to_string(),
                            updated_at: "2024-12-20T15:45:00Z".to_string(),
                            last_login: Some("2024-12-21T09:00:00Z".to_string())
                        }
                    }
                ],
                "links": {
                    "self": "/api/v3/users",
                    "next": "/api/v3/users?page=2"
                },
                "meta": {
                    "total_count": 100,
                    "page": 1,
                    "per_page": 10
                }
            }))
        },
    );

    // Header-based versioning fallback
    app.get(
        "/api/users",
        |req: HttpRequest, res: HttpResponse| async move {
            let version = req.headers.get("accept-version").unwrap_or("2");

            match version {
                "1" => res.set_header("X-API-Deprecated", "true").json(json!({
                    "version": "v1",
                    "message": "V1 via header versioning (deprecated)"
                })),
                "2" => res.set_header("X-API-Version", "2.0").json(json!({
                    "version": "v2",
                    "message": "V2 via header versioning"
                })),
                "3" => res.set_header("X-API-Version", "3.0-beta").json(json!({
                    "version": "v3",
                    "message": "V3 via header versioning (beta)"
                })),
                _ => res.status(400).json(json!({
                    "error": "Unsupported API version",
                    "supported_versions": ["1", "2", "3"]
                })),
            }
        },
    );

    println!("🔢 Router Versioning example server starting on http://127.0.0.1:3000");
    println!("\nAPI Versions:");
    println!("  📕 V1 (Deprecated) - /api/v1/*");
    println!("  📗 V2 (Stable)     - /api/v2/*");
    println!("  📘 V3 (Beta)       - /api/v3/*");
    println!("\nTry these curl commands:\n");
    println!("# V1 (deprecated):");
    println!("curl http://127.0.0.1:3000/api/v1/users\n");
    println!("# V2 (stable):");
    println!("curl http://127.0.0.1:3000/api/v2/users\n");
    println!("# V3 (beta):");
    println!("curl http://127.0.0.1:3000/api/v3/users\n");
    println!("# Header-based versioning:");
    println!("curl http://127.0.0.1:3000/api/users -H 'Accept-Version: 2'\n");
    println!("# Create user V2:");
    println!(
        r#"curl -X POST http://127.0.0.1:3000/api/v2/users -H "Content-Type: application/json" -d '{{"first_name":"Charlie","last_name":"Brown","email":"charlie@example.com","phone":null,"created_at":"2024-12-21T00:00:00Z"}}'"#
    );

    app.listen(3000, || {}).await;
    Ok(())
}
