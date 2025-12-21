//! Forms Example
//!
//! Demonstrates form handling in Ripress:
//! - URL-encoded form data
//! - Form validation
//! - Multi-field forms
//! - Form submission handling

use ripress::{
    app::App,
    req::HttpRequest,
    res::HttpResponse,
    types::RouterFns,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Serve HTML form
    app.get("/", |_: HttpRequest,  res: HttpResponse| async move {
        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Form Examples</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        form { background: #f5f5f5; padding: 20px; border-radius: 8px; margin-bottom: 30px; }
        input, textarea, select { width: 100%; padding: 8px; margin: 5px 0 15px; box-sizing: border-box; }
        button { background: #4CAF50; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; }
        button:hover { background: #45a049; }
        .result { background: #e8f5e9; padding: 15px; border-radius: 4px; margin-top: 20px; }
    </style>
</head>
<body>
    <h1>📝 Ripress Form Examples</h1>
    
    <h2>Login Form</h2>
    <form action="/login" method="post">
        <label>Email:</label>
        <input type="email" name="email" required>
        
        <label>Password:</label>
        <input type="password" name="password" required>
        
        <button type="submit">Login</button>
    </form>
    
    <h2>Registration Form</h2>
    <form action="/register" method="post">
        <label>Username:</label>
        <input type="text" name="username" required>
        
        <label>Email:</label>
        <input type="email" name="email" required>
        
        <label>Age:</label>
        <input type="number" name="age" min="18" max="120" required>
        
        <label>Country:</label>
        <select name="country">
            <option value="US">United States</option>
            <option value="UK">United Kingdom</option>
            <option value="CA">Canada</option>
        </select>
        
        <button type="submit">Register</button>
    </form>
    
    <h2>Contact Form</h2>
    <form action="/contact" method="post">
        <label>Name:</label>
        <input type="text" name="name" required>
        
        <label>Email:</label>
        <input type="email" name="email" required>
        
        <label>Subject:</label>
        <input type="text" name="subject" required>
        
        <label>Message:</label>
        <textarea name="message" rows="5" required></textarea>
        
        <button type="submit">Send Message</button>
    </form>
</body>
</html>
        "#;
        
        res.html(html)
    });

    // Handle login form
    app.post("/login", |req: HttpRequest,  res: HttpResponse| async move {
        let email = req.form_data().unwrap().get("email").unwrap_or("unknown");
        let password = req.form_data().unwrap().get("password").unwrap_or("");

        // Basic validation
        if email.is_empty() || password.is_empty() {
            return res.status(400).json(json!({
                "error": "Email and password are required"
            }));
        }

        // Mock authentication
        if password.len() < 8 {
            return res.status(401).json(json!({
                "error": "Password must be at least 8 characters"
            }));
        }

        res.json(json!({
            "success": true,
            "message": "Login successful",
            "user": {
                "email": email
            }
        }))
    });

    // Handle registration form
    app.post("/register", |req: HttpRequest,  res: HttpResponse| async move {
        let username = req.form_data().unwrap().get("username").unwrap_or("").to_string();
        let email = req.form_data().unwrap().get("email").unwrap_or("").to_string();
        let age_str = req.form_data().unwrap().get("age").unwrap_or("0");
        let country = req.form_data().unwrap().get("country").unwrap_or("Unknown").to_string();

        // Validation
        if username.len() < 3 {
            return res.status(400).json(json!({
                "error": "Username must be at least 3 characters"
            }));
        }

        if !email.contains('@') {
            return res.status(400).json(json!({
                "error": "Invalid email format"
            }));
        }

        let age: u32 = age_str.parse().unwrap_or(0);
        if age < 18 {
            return res.status(400).json(json!({
                "error": "Must be 18 or older to register"
            }));
        }

        res.status(201).json(json!({
            "success": true,
            "message": "Registration successful",
            "user": {
                "username": username,
                "email": email,
                "age": age,
                "country": country
            }
        }))
    });

    // Handle contact form
    app.post("/contact", |req: HttpRequest,  res: HttpResponse| async move {
        let name = req.form_data().unwrap().get("name").unwrap_or("Anonymous");
        let email = req.form_data().unwrap().get("email").unwrap_or("");
        let subject = req.form_data().unwrap().get("subject").unwrap_or("(No subject)");
        let message = req.form_data().unwrap().get("message").unwrap_or("");

        // Validation
        if message.len() < 10 {
            return res.status(400).json(json!({
                "error": "Message must be at least 10 characters"
            }));
        }

        println!("📧 Contact form submitted:");
        println!("  From: {} <{}>", name, email);
        println!("  Subject: {}", subject);
        println!("  Message: {}", message);

        res.json(json!({
            "success": true,
            "message": "Your message has been sent successfully!",
            "details": {
                "name": name,
                "email": email,
                "subject": subject
            }
        }))
    });

    // API endpoint for form data
    app.post("/api/submit", |req: HttpRequest,  res: HttpResponse| async move {
        let mut data = serde_json::Map::new();
        
        for (key, value) in req.form_data().unwrap().iter() {
            data.insert(key.to_string(), json!(value));
        }

        res.json(json!({
            "received": data,
            "field_count": req.form_data().unwrap().len()
        }))
    });

    println!("📝 Forms example server starting on http://127.0.0.1:3000");
    println!("\nOpen http://127.0.0.1:3000 in your browser to see the forms\n");
    println!("Or use curl to test:\n");
    println!(r#"curl -X POST http://127.0.0.1:3000/login -d "email=user@example.com&password=secret123""#);
    println!();
    println!(r#"curl -X POST http://127.0.0.1:3000/register -d "username=john&email=john@example.com&age=25&country=US""#);

    app.listen(3000, || {}).await;
    Ok(())
}