//! Complete CRUD API Example
//!
//! A full REST API with:
//! - Create, Read, Update, Delete operations
//! - In-memory data store
//! - Error handling
//! - Input validation
//! - RESTful routes

use ripress::{
    app::App,
    req::{body::json_data::JsonBody, route_params::Params, HttpRequest},
    res::HttpResponse,
    types::RouterFns,
};
use ripress_derive::{FromJson, FromParams};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
    age: u32,
}

#[derive(Debug, Deserialize, FromJson)]
struct CreateUserInput {
    name: String,
    email: String,
    age: u32,
}

#[derive(Debug, Deserialize, FromJson)]
struct UpdateUserInput {
    name: Option<String>,
    email: Option<String>,
    age: Option<u32>,
}

#[derive(Debug, FromParams)]
struct UserId {
    id: u32,
}

type UserStore = Arc<Mutex<Vec<User>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // In-memory user store
    let users: UserStore = Arc::new(Mutex::new(vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 25,
        },
    ]));

    // GET /users - List all users
    let users_clone = users.clone();
    app.get("/users", move |_: HttpRequest, res: HttpResponse| {
        let users = users_clone.clone();
        async move {
            let users = users.lock().unwrap();
            res.json(json!({
                "users": *users,
                "count": users.len()
            }))
        }
    });

    // GET /users/:id - Get single user
    let users_clone = users.clone();
    app.get(
        "/users/:id",
        move |params: Params<UserId>, res: HttpResponse| {
            let users = users_clone.clone();
            async move {
                let users = users.lock().unwrap();

                match users.iter().find(|u| u.id == params.id) {
                    Some(user) => res.json(json!({"user": user})),
                    None => res.status(404).json(json!({
                        "error": "User not found",
                        "id": params.id
                    })),
                }
            }
        },
    );

    // POST /users - Create new user
    let users_clone = users.clone();
    app.post(
        "/users",
        move |body: JsonBody<CreateUserInput>, res: HttpResponse| {
            let users = users_clone.clone();
            async move {
                // Validation
                if body.name.trim().is_empty() {
                    return res.status(400).json(json!({
                        "error": "Name is required"
                    }));
                }

                if !body.email.contains('@') {
                    return res.status(400).json(json!({
                        "error": "Invalid email format"
                    }));
                }

                if body.age < 18 {
                    return res.status(400).json(json!({
                        "error": "Age must be 18 or older"
                    }));
                }

                let mut users = users.lock().unwrap();

                // Check if email already exists
                if users.iter().any(|u| u.email == body.email) {
                    return res.status(409).json(json!({
                        "error": "Email already exists"
                    }));
                }

                let new_id = users.iter().map(|u| u.id).max().unwrap_or(0) + 1;
                let new_user = User {
                    id: new_id,
                    name: body.name.clone(),
                    email: body.email.clone(),
                    age: body.age,
                };

                users.push(new_user.clone());

                res.status(201).json(json!({
                    "message": "User created successfully",
                    "user": new_user
                }))
            }
        },
    );

    // PUT /users/:id - Update user
    let users_clone = users.clone();
    app.put(
        "/users/:id",
        move |(params, body): (Params<UserId>, JsonBody<UpdateUserInput>), res: HttpResponse| {
            let users = users_clone.clone();
            async move {
                let mut users = users.lock().unwrap();

                match users.iter_mut().find(|u| u.id == params.id) {
                    Some(user) => {
                        if let Some(name) = &body.name {
                            if name.trim().is_empty() {
                                return res.status(400).json(json!({
                                    "error": "Name cannot be empty"
                                }));
                            }
                            user.name = name.clone();
                        }

                        if let Some(email) = &body.email {
                            if !email.contains('@') {
                                return res.status(400).json(json!({
                                    "error": "Invalid email format"
                                }));
                            }
                            user.email = email.clone();
                        }

                        if let Some(age) = body.age {
                            if age < 18 {
                                return res.status(400).json(json!({
                                    "error": "Age must be 18 or older"
                                }));
                            }
                            user.age = age;
                        }

                        res.json(json!({
                            "message": "User updated successfully",
                            "user": user
                        }))
                    }
                    None => res.status(404).json(json!({
                        "error": "User not found",
                        "id": params.id
                    })),
                }
            }
        },
    );

    // DELETE /users/:id - Delete user
    let users_clone = users.clone();
    app.delete(
        "/users/:id",
        move |params: Params<UserId>, res: HttpResponse| {
            let users = users_clone.clone();
            async move {
                let mut users = users.lock().unwrap();
                let initial_len = users.len();

                users.retain(|u| u.id != params.id);

                if users.len() < initial_len {
                    res.status(200).json(json!({
                        "message": "User deleted successfully",
                        "id": params.id
                    }))
                } else {
                    res.status(404).json(json!({
                        "error": "User not found",
                        "id": params.id
                    }))
                }
            }
        },
    );

    // Health check
    app.get("/health", |_: HttpRequest, res: HttpResponse| async move {
        res.json(json!({
            "status": "healthy",
            "service": "User API"
        }))
    });

    println!("🚀 CRUD API example server starting on http://127.0.0.1:3000");
    println!("\nAvailable endpoints:\n");
    println!("GET    /users       - List all users");
    println!("GET    /users/:id   - Get user by ID");
    println!("POST   /users       - Create new user");
    println!("PUT    /users/:id   - Update user");
    println!("DELETE /users/:id   - Delete user");
    println!("GET    /health      - Health check\n");
    println!("Try these curl commands:\n");
    println!("# List all users");
    println!("curl http://127.0.0.1:3000/users\n");
    println!("# Get user by ID");
    println!("curl http://127.0.0.1:3000/users/1\n");
    println!("# Create new user");
    println!(
        r#"curl -X POST http://127.0.0.1:3000/users -H "Content-Type: application/json" -d '{{"name":"Charlie","email":"charlie@example.com","age":28}}'"#
    );
    println!("\n# Update user");
    println!(
        r#"curl -X PUT http://127.0.0.1:3000/users/1 -H "Content-Type: application/json" -d '{{"age":31}}'"#
    );
    println!("\n# Delete user");
    println!("curl -X DELETE http://127.0.0.1:3000/users/2\n");

    app.listen(3000, || {}).await;
    Ok(())
}
