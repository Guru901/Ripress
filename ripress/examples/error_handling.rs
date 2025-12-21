//! Error Handling Example
//!
//! Demonstrates comprehensive error handling:
//! - Custom error types
//! - Error middleware
//! - Validation errors
//! - Database simulation errors
//! - Graceful error responses

use ripress::{
    app::App,
    req::{body::json_data::JsonBody, route_params::Params, HttpRequest},
    res::HttpResponse,
    types::RouterFns,
};
use ripress_derive::FromJson;
use serde::Deserialize;
use serde_json::json;
use std::fmt;

// Custom error types
#[derive(Debug, Clone)]
enum AppError {
    NotFound(String),
    ValidationError(String),
    DatabaseError(String),
    Unauthorized(String),
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}

impl AppError {
    fn status_code(&self) -> u16 {
        match self {
            AppError::NotFound(_) => 404,
            AppError::ValidationError(_) => 400,
            AppError::DatabaseError(_) => 503,
            AppError::Unauthorized(_) => 401,
            AppError::InternalError(_) => 500,
        }
    }

    fn error_type(&self) -> &str {
        match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::InternalError(_) => "INTERNAL_ERROR",
        }
    }

    fn to_response(&self) -> HttpResponse {
        HttpResponse::new().status(self.status_code()).json(json!({
            "error": {
                "type": self.error_type(),
                "message": self.to_string(),
                "status": self.status_code()
            }
        }))
    }
}

// Request structures
#[derive(Debug, Deserialize)]
struct CreateUserInput {
    name: String,
    email: String,
    age: u32,
}

impl ripress::req::body::json_data::FromJson for CreateUserInput {
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

// Validation helper
fn validate_user_input(input: &CreateUserInput) -> Result<(), AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Name cannot be empty".to_string(),
        ));
    }

    if input.name.len() < 2 {
        return Err(AppError::ValidationError(
            "Name must be at least 2 characters".to_string(),
        ));
    }

    if !input.email.contains('@') || !input.email.contains('.') {
        return Err(AppError::ValidationError(
            "Invalid email format".to_string(),
        ));
    }

    if input.age < 18 {
        return Err(AppError::ValidationError(
            "Age must be 18 or older".to_string(),
        ));
    }

    if input.age > 150 {
        return Err(AppError::ValidationError("Invalid age value".to_string()));
    }

    Ok(())
}

// Simulated database operations
fn simulate_database_operation(success: bool) -> Result<(), AppError> {
    if success {
        Ok(())
    } else {
        Err(AppError::DatabaseError(
            "Failed to connect to database".to_string(),
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Global error handling middleware
    app.use_post_middleware(None, move |req: HttpRequest, res| {
        Box::pin(async move {
            if res.status_code() >= 400 {
                println!("❌ Error Response: {}", res.status_code());
            }

            (req, Some(res))
        })
    });

    // Home endpoint
    app.get("/", |_: HttpRequest, res: HttpResponse| async move {
        res.json(json!({
            "message": "Error Handling Example",
            "endpoints": {
                "validation": "/api/users (POST)",
                "not_found": "/api/users/999",
                "unauthorized": "/api/admin",
                "database_error": "/api/db-fail",
                "internal_error": "/api/panic",
                "custom_error": "/api/custom-error"
            }
        }))
    });

    // Validation error example
    app.post(
        "/api/users",
        |body: JsonBody<CreateUserInput>, res: HttpResponse| async move {
            // Validate input
            if let Err(err) = validate_user_input(&body) {
                return err.to_response();
            }

            res.status(201).json(json!({
                "message": "User created successfully",
                "user": {
                    "name": body.name,
                    "email": body.email,
                    "age": body.age
                }
            }))
        },
    );

    // Not found error example
    app.get(
        "/api/users/:id",
        |params: Params<UserId>, res: HttpResponse| async move {
            if params.id > 100 {
                return AppError::NotFound(format!("User with id {} not found", params.id))
                    .to_response();
            }

            res.json(json!({
                "user": {
                    "id": params.id,
                    "name": "John Doe",
                    "email": "john@example.com"
                }
            }))
        },
    );

    // Unauthorized error example
    app.get(
        "/api/admin",
        |req: HttpRequest, res: HttpResponse| async move {
            let auth = req.headers.get("authorization");

            match auth {
                Some(token) if token == "Bearer admin_token" => res.json(json!({
                    "message": "Welcome to admin panel"
                })),
                _ => AppError::Unauthorized("Admin access required".to_string()).to_response(),
            }
        },
    );

    // Database error example
    app.get(
        "/api/db-fail",
        |_: HttpRequest, _res: HttpResponse| async move {
            let result = simulate_database_operation(false);

            match result {
                Ok(_) => HttpResponse::new().json(json!({"message": "Success"})),
                Err(err) => err.to_response(),
            }
        },
    );

    // Internal error example (panic recovery)
    app.get(
        "/api/panic",
        |_: HttpRequest, res: HttpResponse| async move {
            // In a real application, you'd have panic recovery middleware
            res.status(500).json(json!({
                "error": {
                    "type": "INTERNAL_ERROR",
                    "message": "An unexpected error occurred",
                    "status": 500
                }
            }))
        },
    );

    // Custom error demonstration
    app.get(
        "/api/custom-error",
        |_: HttpRequest, _res: HttpResponse| async move {
            let errors = vec![
                AppError::NotFound("Resource not found".to_string()),
                AppError::ValidationError("Invalid input".to_string()),
                AppError::DatabaseError("Connection failed".to_string()),
                AppError::Unauthorized("Access denied".to_string()),
                AppError::InternalError("Something went wrong".to_string()),
            ];

            // Pick an error based on current system time milliseconds, just for demo, no rand required
            let idx = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos() as usize)
                .unwrap_or(0))
                % errors.len();
            let random_error = &errors[idx];
            random_error.clone().to_response()
        },
    );

    // Error with additional context
    #[derive(Deserialize, Debug, FromJson)]
    struct ProcessInput {
        action: String,
    }

    app.post(
        "/api/process",
        |body: JsonBody<ProcessInput>, res: HttpResponse| async move {
            let input = &*body;

            res.json(json!({
                "message": "Processing completed",
                "action": input.action
            }))
        },
    );

    // Global error fallback (404)
    app.get("/*", |req: HttpRequest, _res: HttpResponse| async move {
        AppError::NotFound(format!("Route {} not found", req.origin_url)).to_response()
    });

    println!("⚠️  Error Handling example server starting on http://127.0.0.1:3000");
    println!("\nTest different error scenarios:\n");
    println!("# Validation error (empty name):");
    println!(
        r#"curl -X POST http://127.0.0.1:3000/api/users -H "Content-Type: application/json" -d '{{"name":"","email":"test@example.com","age":25}}'"#
    );
    println!("\n# Validation error (invalid email):");
    println!(
        r#"curl -X POST http://127.0.0.1:3000/api/users -H "Content-Type: application/json" -d '{{"name":"John","email":"invalid","age":25}}'"#
    );
    println!("\n# Not found error:");
    println!("curl http://127.0.0.1:3000/api/users/999");
    println!("\n# Unauthorized error:");
    println!("curl http://127.0.0.1:3000/api/admin");
    println!("\n# Database error:");
    println!("curl http://127.0.0.1:3000/api/db-fail");
    println!("\n# Random error:");
    println!("curl http://127.0.0.1:3000/api/custom-error");

    app.listen(3000, || {}).await;
    Ok(())
}
