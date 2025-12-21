//! Extractors Example
//!
//! This example demonstrates how to use multiple extractors in Ripress handlers:
//! - Route parameters (Params)
//! - Query parameters (QueryParam)
//! - JSON body (JsonBody)
//! - Headers (Headers)
//! - Combining multiple extractors

use ripress::{
    app::App,
    req::{
        body::json_data::JsonBody, query_params::QueryParam, request_headers::Headers,
        route_params::Params,
    },
    res::HttpResponse,
    types::RouterFns,
};
use ripress_derive::{FromJson, FromParams, FromQueryParam};
use serde::{Deserialize, Serialize};
use serde_json::json;

// Route parameter extractor
#[derive(Debug, FromParams)]
struct UserId {
    id: u32,
}

#[derive(Debug, FromParams)]
struct PostSlug {
    slug: String,
}

// Query parameter extractor
#[derive(Debug, FromQueryParam)]
struct Pagination {
    page: usize,
    per_page: usize,
}

// JSON body structures
#[derive(Debug, Deserialize, Serialize, FromJson)]
struct CreateUser {
    name: String,
    email: String,
    age: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdatePost {
    title: Option<String>,
    content: Option<String>,
    published: Option<bool>,
}

impl ripress::req::body::json_data::FromJson for UpdatePost {
    fn from_json(data: &ripress::req::body::RequestBodyContent) -> Result<Self, String> {
        if let ripress::req::body::RequestBodyContent::JSON(json_val) = data {
            serde_json::from_value(json_val.clone()).map_err(|e| e.to_string())
        } else {
            Err("Expected JSON body".to_string())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Example 1: Single extractor - Route parameter
    app.get(
        "/users/:id",
        |params: Params<UserId>, res: HttpResponse| async move {
            println!("Fetching user with id: {}", params.id);
            res.json(json!({
                "user_id": params.id,
                "name": "John Doe",
                "email": "john@example.com"
            }))
        },
    );

    // Example 2: Single extractor - Query parameters
    app.get(
        "/users",
        |query: QueryParam<Pagination>, res: HttpResponse| async move {
            println!("Page: {}, Per page: {}", query.page, query.per_page);
            res.json(json!({
                "page": query.page,
                "per_page": query.per_page,
                "users": [
                    {"id": 1, "name": "Alice"},
                    {"id": 2, "name": "Bob"}
                ]
            }))
        },
    );

    // Example 3: Single extractor - JSON body
    app.post(
        "/users",
        |body: JsonBody<CreateUser>, res: HttpResponse| async move {
            println!("Creating user: {:?}", body.name);
            res.status(201).json(json!({
                "message": "User created",
                "user": {
                    "name": body.name,
                    "email": body.email,
                    "age": body.age
                }
            }))
        },
    );

    // Example 4: Two extractors - Route param + JSON body
    app.put(
        "/users/:id",
        |(params, body): (Params<UserId>, JsonBody<CreateUser>), res: HttpResponse| async move {
            println!("Updating user {} with data: {:?}", params.id, body.name);
            res.json(json!({
                "message": "User updated",
                "user_id": params.id,
                "updated_data": {
                    "name": body.name,
                    "email": body.email
                }
            }))
        },
    );

    // Example 5: Three extractors - Route param + Query + Headers
    app.get(
        "/posts/:slug",
        |(params, query, headers): (Params<PostSlug>, QueryParam<Pagination>, Headers),
         res: HttpResponse| async move {
            let auth = headers.get("authorization").unwrap_or("none");

            println!(
                "Fetching post: {}, Page: {}, Auth: {}",
                params.slug, query.page, auth
            );

            res.json(json!({
                "post": {
                    "slug": params.slug,
                    "title": "Sample Post",
                    "content": "..."
                },
                "pagination": {
                    "page": query.page,
                    "per_page": query.per_page
                },
                "authenticated": auth != "none"
            }))
        },
    );

    // Example 6: Four extractors - All combined
    app.put(
        "/api/posts/:slug",
        |(params, query, body, headers): (
            Params<PostSlug>,
            QueryParam<Pagination>,
            JsonBody<UpdatePost>,
            Headers,
        ),
         res: HttpResponse| async move {
            let user_agent = headers.get("user-agent").unwrap_or("unknown");

            println!("Updating post: {}", params.slug);
            println!(
                "Pagination: page={}, per_page={}",
                query.page, query.per_page
            );
            println!("User agent: {}", user_agent);

            res.json(json!({
                "message": "Post updated",
                "slug": params.slug,
                "updates": {
                    "title": body.title,
                    "content": body.content,
                    "published": body.published
                },
                "metadata": {
                    "page": query.page,
                    "user_agent": user_agent
                }
            }))
        },
    );

    // Example 7: Headers extractor
    app.get(
        "/debug/headers",
        |headers: Headers, res: HttpResponse| async move {
            let mut header_map = serde_json::Map::new();

            for (name, value) in headers.iter() {
                if let Ok(val_str) = value.to_str() {
                    header_map.insert(name.to_string(), json!(val_str));
                }
            }

            res.json(json!({
                "headers": header_map
            }))
        },
    );

    // Example 8: Delete with route param only
    app.delete(
        "/users/:id",
        |params: Params<UserId>, res: HttpResponse| async move {
            println!("Deleting user with id: {}", params.id);
            res.status(204).text("")
        },
    );

    println!("🚀 Extractors example server starting on http://127.0.0.1:3000");
    println!("\nTry these curl commands:\n");
    println!("📌 Route parameter:");
    println!("curl http://127.0.0.1:3000/users/123");
    println!("\n🔍 Query parameters:");
    println!("curl 'http://127.0.0.1:3000/users?page=2&per_page=20'");
    println!("\n📝 JSON body:");
    println!(
        r#"curl -X POST http://127.0.0.1:3000/users -H "Content-Type: application/json" -d '{{"name":"Alice","email":"alice@example.com","age":30}}'"#
    );
    println!("\n🔄 Route param + JSON body:");
    println!(
        r#"curl -X PUT http://127.0.0.1:3000/users/42 -H "Content-Type: application/json" -d '{{"name":"Bob","email":"bob@example.com","age":35}}'"#
    );
    println!("\n🎯 Multiple extractors:");
    println!(
        r#"curl 'http://127.0.0.1:3000/posts/my-awesome-post?page=1' -H "Authorization: Bearer token123""#
    );
    println!("\n🔧 Headers:");
    println!("curl http://127.0.0.1:3000/debug/headers -H 'X-Custom-Header: test'");

    app.listen(3000, || {}).await;
    Ok(())
}
