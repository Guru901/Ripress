use std::sync::{Arc, Mutex};

use ripress::app::App;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Clone)]
struct Blog {
    title: String,
    content: String,
    author: String,
}

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let blogs: Arc<Mutex<Vec<Blog>>> = Arc::new(Mutex::new(Vec::new()));
    let blogs_clone = Arc::clone(&blogs);

    app.get("/api/blog", move |_, res| {
        let blogs = blogs_clone.lock().unwrap().clone();
        async move { res.ok().json(blogs) }
    });

    let blogs_clone = Arc::clone(&blogs);

    app.post("/api/blog", move |req, res| {
        let blogs = blogs_clone.clone();
        async move {
            let data = req.json::<Blog>();

            match data {
                Ok(blog) => {
                    blogs.lock().unwrap().push(blog);
                    res.ok().json(json!({
                        "message": "Blog added successfully",
                    }))
                }
                Err(err) => res.bad_request().text(err.to_string()),
            }
        }
    });

    let blogs_clone = Arc::clone(&blogs);
    app.put("/api/blog", move |req, res| {
        let blogs = blogs_clone.clone();
        async move {
            let data = req.json::<Blog>();
            match data {
                Ok(blog) => {
                    let mut blogs = blogs.lock().unwrap();
                    let blog_clone = blog.clone();
                    for (i, b) in blogs.iter().enumerate() {
                        if b.title == blog_clone.title {
                            blogs[i] = blog;
                            break;
                        }
                    }

                    return res.ok().json(json!({
                        "message": "Blog updated successfully",
                    }));
                }
                Err(err) => return res.bad_request().text(err.to_string()),
            }
        }
    });

    let blogs_clone = Arc::clone(&blogs);
    app.delete("/api/blog", move |req, res| {
        let blogs = blogs_clone.clone();
        async move {
            let data = req.json::<Blog>();
            match data {
                Ok(blog) => {
                    let mut blogs = blogs.lock().unwrap();
                    let blog_clone = blog.clone();
                    for (i, b) in blogs.iter().enumerate() {
                        if b.title == blog_clone.title {
                            blogs.remove(i);
                            break;
                        }
                    }
                    return res.ok().json(json!({
                        "message": "Blog deleted successfully",
                    }));
                }
                Err(err) => return res.bad_request().text(err.to_string()),
            }
        }
    });

    app.listen(8080, || {
        println!("Server running on http://localhost:8080");
    })
    .await;
}
