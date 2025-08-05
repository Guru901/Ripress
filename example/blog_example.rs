use ripress::app::App;
use ripress::types::RouterFns;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone)]
struct Post {
    id: Option<u32>,
    title: String,
    content: String,
    author: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

type PostStorage = Arc<Mutex<HashMap<u32, Post>>>;

fn create_error_response(error: &str, message: &str) -> Value {
    json!(ErrorResponse {
        error: error.to_string(),
        message: message.to_string(),
    })
}

#[tokio::main]
async fn main() {
    // Use a proper data structure for better performance and thread safety
    let posts: PostStorage = Arc::new(Mutex::new(HashMap::new()));
    let next_id = Arc::new(Mutex::new(1u32));

    // Initialize with sample data
    {
        let mut posts_lock = posts.lock().unwrap();
        let mut id_lock = next_id.lock().unwrap();

        posts_lock.insert(
            1,
            Post {
                id: Some(1),
                title: "First Post".to_string(),
                content: "This is my first blog post".to_string(),
                author: "john_doe".to_string(),
            },
        );

        posts_lock.insert(
            2,
            Post {
                id: Some(2),
                title: "Ripress Tutorial".to_string(),
                content: "Learning Ripress is fun".to_string(),
                author: "guru".to_string(),
            },
        );

        *id_lock = 3;
    }

    let mut app = App::new();

    // GET /api/posts - List all posts with optional filtering
    let posts_clone = posts.clone();
    app.get("/api/posts", move |req, res| {
        let posts = posts_clone.clone();
        async move {
            let posts_lock = match posts.lock() {
                Ok(lock) => lock,
                Err(_) => {
                    return res.status(500).json(create_error_response(
                        "internal_error",
                        "Failed to access posts data",
                    ));
                }
            };

            let mut filtered_posts: Vec<Post> = posts_lock.values().cloned().collect();

            // Filter by author if provided
            if let Ok(author) = req.get_query("author") {
                filtered_posts.retain(|post| post.author == author);
            }

            // Apply limit if provided
            if let Ok(limit_str) = req.get_query("limit") {
                if let Ok(limit) = limit_str.parse::<usize>() {
                    filtered_posts.truncate(limit);
                }
            }

            // Sort by id for consistent ordering
            filtered_posts.sort_by_key(|post| post.id.unwrap_or(0));

            res.ok().json(filtered_posts)
        }
    });

    // GET /api/posts/:id - Get a specific post
    let posts_clone = posts.clone();
    app.get("/api/posts/:id", move |req, res| {
        let posts = posts_clone.clone();
        async move {
            let id_str = match req.get_params("id") {
                Ok(id) => id,
                Err(_) => {
                    return res.status(400).json(create_error_response(
                        "invalid_request",
                        "Post ID is required",
                    ));
                }
            };

            let id: u32 = match id_str.parse() {
                Ok(id) => id,
                Err(_) => {
                    return res.status(400).json(create_error_response(
                        "invalid_request",
                        "Post ID must be a valid number",
                    ));
                }
            };

            let posts_lock = match posts.lock() {
                Ok(lock) => lock,
                Err(_) => {
                    return res.status(500).json(create_error_response(
                        "internal_error",
                        "Failed to access posts data",
                    ));
                }
            };

            match posts_lock.get(&id) {
                Some(post) => res.ok().json(post),
                None => res.status(404).json(create_error_response(
                    "not_found",
                    &format!("Post with ID {} not found", id),
                )),
            }
        }
    });

    // POST /api/posts - Create a new post
    let posts_clone = posts.clone();
    let next_id_clone = next_id.clone();
    app.post("/api/posts", move |req, res| {
        let posts = posts_clone.clone();
        let next_id = next_id_clone.clone();
        async move {
            let mut new_post: Post = match req.json() {
                Ok(post) => post,
                Err(_) => {
                    return res.status(400).json(create_error_response(
                        "invalid_request",
                        "Invalid JSON data or missing required fields",
                    ));
                }
            };

            // Validate required fields
            if new_post.title.trim().is_empty() {
                return res.status(400).json(create_error_response(
                    "validation_error",
                    "Title cannot be empty",
                ));
            }

            if new_post.author.trim().is_empty() {
                return res.status(400).json(create_error_response(
                    "validation_error",
                    "Author cannot be empty",
                ));
            }

            let (mut posts_lock, mut id_lock) = match (posts.lock(), next_id.lock()) {
                (Ok(posts), Ok(id)) => (posts, id),
                _ => {
                    return res.status(500).json(create_error_response(
                        "internal_error",
                        "Failed to access data storage",
                    ));
                }
            };

            let post_id = *id_lock;
            *id_lock += 1;

            new_post.id = Some(post_id);
            posts_lock.insert(post_id, new_post.clone());

            res.status(201).json(new_post)
        }
    });

    // PUT /api/posts/:id - Update an existing post
    let posts_clone = posts.clone();
    app.put("/api/posts/:id", move |req, res| {
        let posts = posts_clone.clone();
        async move {
            let id_str = match req.get_params("id") {
                Ok(id) => id,
                Err(_) => {
                    return res.status(400).json(create_error_response(
                        "invalid_request",
                        "Post ID is required",
                    ));
                }
            };

            let id: u32 = match id_str.parse() {
                Ok(id) => id,
                Err(_) => {
                    return res.status(400).json(create_error_response(
                        "invalid_request",
                        "Post ID must be a valid number",
                    ));
                }
            };

            let updated_post: Post = match req.json() {
                Ok(post) => post,
                Err(_) => {
                    return res.status(400).json(create_error_response(
                        "invalid_request",
                        "Invalid JSON data",
                    ));
                }
            };

            // Validate required fields
            if updated_post.title.trim().is_empty() {
                return res.status(400).json(create_error_response(
                    "validation_error",
                    "Title cannot be empty",
                ));
            }

            if updated_post.author.trim().is_empty() {
                return res.status(400).json(create_error_response(
                    "validation_error",
                    "Author cannot be empty",
                ));
            }

            let mut posts_lock = match posts.lock() {
                Ok(lock) => lock,
                Err(_) => {
                    return res.status(500).json(create_error_response(
                        "internal_error",
                        "Failed to access posts data",
                    ));
                }
            };

            match posts_lock.get_mut(&id) {
                Some(post) => {
                    post.title = updated_post.title;
                    post.content = updated_post.content;
                    post.author = updated_post.author;
                    res.ok().json(post.clone())
                }
                None => res.status(404).json(create_error_response(
                    "not_found",
                    &format!("Post with ID {} not found", id),
                )),
            }
        }
    });

    // DELETE /api/posts/:id - Delete a post
    let posts_clone = posts.clone();
    app.delete("/api/posts/:id", move |req, res| {
        let posts = posts_clone.clone();
        async move {
            let id_str = match req.get_params("id") {
                Ok(id) => id,
                Err(_) => {
                    return res.status(400).json(create_error_response(
                        "invalid_request",
                        "Post ID is required",
                    ));
                }
            };

            let id: u32 = match id_str.parse() {
                Ok(id) => id,
                Err(_) => {
                    return res.status(400).json(create_error_response(
                        "invalid_request",
                        "Post ID must be a valid number",
                    ));
                }
            };

            let mut posts_lock = match posts.lock() {
                Ok(lock) => lock,
                Err(_) => {
                    return res.status(500).json(create_error_response(
                        "internal_error",
                        "Failed to access posts data",
                    ));
                }
            };

            match posts_lock.remove(&id) {
                Some(deleted_post) => res.ok().json(json!({
                    "message": "Post deleted successfully",
                    "deleted_post": deleted_post
                })),
                None => res.status(404).json(create_error_response(
                    "not_found",
                    &format!("Post with ID {} not found", id),
                )),
            }
        }
    });

    app.listen(8080, || {
        println!("Blog API server starting...");
        println!("Blog API server started on port 8080");
        println!("Available endpoints:");
        println!(
            "  GET    /api/posts           - List all posts (supports ?author=<name>&limit=<num>)"
        );
        println!("  GET    /api/posts/:id       - Get specific post");
        println!("  POST   /api/posts           - Create new post");
        println!("  PUT    /api/posts/:id       - Update existing post");
        println!("  DELETE /api/posts/:id       - Delete post");
    })
    .await
    .unwrap();
}
