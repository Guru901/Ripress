use ripress::app::App;
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Runs for paths that start with /auth
    app.use_middleware("/auth", |mut req, res, next| {
        return Box::pin(async move {
            // DO your auth here
            if let Ok(token) = req.get_cookie("auth-token") {
                req.set_data("token", token);
                next.run(req, res).await
            } else {
                res.unauthorized().text("Unauthorized")
            }
        });
    });

    // Runs for all paths
    app.use_middleware("", |req, res, next| {
        return Box::pin(async move {
            let res = res.set_header("X-message", "This is addHeader middleware!");
            next.run(req, res).await
        });
    });

    app.get(
        "/",
        |req, res| async move { res.ok().text("Helloo Ripress!!") },
    );

    // Request params
    app.get("/user/{name}", |req, res| async move {
        let username = req.get_params("name").unwrap_or("Unknown");
        res.ok().text(format!("Helloo, {username}"))
    });

    // Authentication required
    app.get("/auth/*", |req, res| async move {
        res.ok().text("You are authorized")
    });

    // Request headers
    app.get("/user-agent", |req, res| async move {
        let user_agent = req.get_header("User-Agent").unwrap_or("Unknown");
        res.text(format!("Your User Agent is {user_agent}"))
    });

    // JSON Response
    app.get("/api/blog", |req, res| async move {
        let posts = json!([
          { "id": 1, "title": "Good Morning" },
          { "id": 2, "title": "Good Afternoon" },
          { "id": 3, "title": "Good Evening" },
          { "id": 4, "title": "Good Night"}
        ]);

        res.json(posts)
    });

    app.post("/api/blog", |req, res| async move {
        res.status(201).json(json!({"message": "blog created"}))
    });

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    }).await;
}
