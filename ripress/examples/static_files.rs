//! Static Files Example
//!
//! Demonstrates serving static files and assets:
//! - HTML pages
//! - CSS stylesheets
//! - JavaScript files
//! - Images
//! - SPA routing

use ripress::req::HttpRequest;
use ripress::{app::App, res::HttpResponse, types::RouterFns};
use std::fs;
use std::path::Path;

fn read_file(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

fn get_content_type(path: &str) -> &'static str {
    let extension = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Create static directory and files if they don't exist
    fs::create_dir_all("static")?;
    fs::create_dir_all("static/css")?;
    fs::create_dir_all("static/js")?;

    // Create index.html
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Ripress Static Files Example</title>
    <link rel="stylesheet" href="/static/css/styles.css">
</head>
<body>
    <div class="container">
        <h1>🎨 Static Files Example</h1>
        <p>This page is served from a static HTML file!</p>
        
        <div class="features">
            <div class="feature">
                <h3>📄 HTML</h3>
                <p>Serving HTML pages</p>
            </div>
            <div class="feature">
                <h3>🎨 CSS</h3>
                <p>Stylesheets working</p>
            </div>
            <div class="feature">
                <h3>⚡ JavaScript</h3>
                <p>Scripts loading</p>
            </div>
        </div>
        
        <button id="testBtn">Click me!</button>
        <div id="output"></div>
        
        <div class="links">
            <a href="/about">About Page</a>
            <a href="/contact">Contact Page</a>
            <a href="/api/data">API Data</a>
        </div>
    </div>
    
    <script src="/static/js/app.js"></script>
</body>
</html>"#;

    // Create styles.css
    let styles_css = r#"* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
    color: #333;
}

.container {
    background: white;
    padding: 40px;
    border-radius: 16px;
    box-shadow: 0 20px 60px rgba(0,0,0,0.3);
    max-width: 800px;
    width: 90%;
}

h1 {
    color: #667eea;
    margin-bottom: 20px;
    text-align: center;
}

.features {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 20px;
    margin: 30px 0;
}

.feature {
    background: #f5f5f5;
    padding: 20px;
    border-radius: 8px;
    text-align: center;
    transition: transform 0.3s;
}

.feature:hover {
    transform: translateY(-5px);
    box-shadow: 0 5px 15px rgba(0,0,0,0.1);
}

.feature h3 {
    color: #764ba2;
    margin-bottom: 10px;
}

button {
    background: #667eea;
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    transition: background 0.3s;
    display: block;
    margin: 20px auto;
}

button:hover {
    background: #5568d3;
}

#output {
    text-align: center;
    margin: 20px 0;
    padding: 10px;
    background: #e8f5e9;
    border-radius: 6px;
    min-height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.links {
    display: flex;
    gap: 15px;
    justify-content: center;
    margin-top: 30px;
}

.links a {
    color: #667eea;
    text-decoration: none;
    padding: 8px 16px;
    border: 2px solid #667eea;
    border-radius: 6px;
    transition: all 0.3s;
}

.links a:hover {
    background: #667eea;
    color: white;
}"#;

    // Create app.js
    let app_js = r#"console.log('✅ JavaScript loaded successfully!');

document.getElementById('testBtn').addEventListener('click', async () => {
    const output = document.getElementById('output');
    output.textContent = 'Loading...';
    
    try {
        const response = await fetch('/api/data');
        const data = await response.json();
        output.textContent = `✅ ${data.message}`;
        output.style.background = '#e8f5e9';
    } catch (error) {
        output.textContent = `❌ Error: ${error.message}`;
        output.style.background = '#ffebee';
    }
});

// Animate features on load
document.addEventListener('DOMContentLoaded', () => {
    const features = document.querySelectorAll('.feature');
    features.forEach((feature, index) => {
        setTimeout(() => {
            feature.style.opacity = '0';
            feature.style.transform = 'translateY(20px)';
            feature.style.transition = 'all 0.5s';
            
            setTimeout(() => {
                feature.style.opacity = '1';
                feature.style.transform = 'translateY(0)';
            }, 50);
        }, index * 100);
    });
});"#;

    fs::write("static/index.html", index_html)?;
    fs::write("static/css/styles.css", styles_css)?;
    fs::write("static/js/app.js", app_js)?;

    // Serve index page
    app.get("/", |_: HttpRequest, res: HttpResponse| async move {
        match read_file("static/index.html") {
            Ok(content) => res.set_header("content-type", "text/html").text(&content),
            Err(_) => res.status(404).text("File not found"),
        }
    });

    // Serve static files
    app.get(
        "/static/*",
        |req: ripress::req::HttpRequest, res: HttpResponse| async move {
            let path = req.origin_url.to_string();
            let file_path = &path[1..]; // Remove leading slash

            match read_file(file_path) {
                Ok(content) => {
                    let content_type = get_content_type(file_path);
                    res.set_header("content-type", content_type)
                        .set_header("cache-control", "public, max-age=3600")
                        .text(&content)
                }
                Err(_) => res.status(404).text("File not found"),
            }
        },
    );

    // API endpoint
    app.get(
        "/api/data",
        |_: HttpRequest, res: HttpResponse| async move {
            let now = std::time::SystemTime::now();
            let timestamp_str = match now.duration_since(std::time::UNIX_EPOCH) {
                Ok(dur) => {
                    let secs = dur.as_secs() as i64;
                    let nsecs = dur.subsec_nanos();
                    format!("{}.{}Z", secs, nsecs)
                }
                Err(_) => "unknown".to_string(),
            };

            res.json(serde_json::json!({
                "message": "Data loaded from API!",
                "timestamp": timestamp_str,
                "status": "success"
            }))
        },
    );

    // Additional HTML pages
    app.get("/about", |_: HttpRequest, res: HttpResponse| async move {
        let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>About - Ripress</title>
    <link rel="stylesheet" href="/static/css/styles.css">
</head>
<body>
    <div class="container">
        <h1>📖 About This Example</h1>
        <p>This demonstrates serving static files in Ripress framework.</p>
        <div class="links">
            <a href="/">Home</a>
            <a href="/contact">Contact</a>
        </div>
    </div>
</body>
</html>"#;
        res.html(html)
    });

    app.get("/contact", |_: HttpRequest, res: HttpResponse| async move {
        let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Contact - Ripress</title>
    <link rel="stylesheet" href="/static/css/styles.css">
</head>
<body>
    <div class="container">
        <h1>📧 Contact Us</h1>
        <p>Get in touch with the Ripress team!</p>
        <div class="links">
            <a href="/">Home</a>
            <a href="/about">About</a>
        </div>
    </div>
</body>
</html>"#;
        res.html(html)
    });

    println!("📁 Static Files example server starting on http://127.0.0.1:3000");
    println!("\nStatic files created in ./static directory");
    println!("\nAvailable routes:");
    println!("  GET  /                    - Home page (HTML)");
    println!("  GET  /static/css/styles.css - Stylesheet");
    println!("  GET  /static/js/app.js   - JavaScript");
    println!("  GET  /about              - About page");
    println!("  GET  /contact            - Contact page");
    println!("  GET  /api/data           - API endpoint");
    println!("\nOpen http://127.0.0.1:3000 in your browser!");

    app.listen(3000, || {}).await;
    Ok(())
}
