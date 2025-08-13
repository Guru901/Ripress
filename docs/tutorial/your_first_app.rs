use ripress::{app::App, types::RouterFns};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
struct Todo {
    id: i32,
    content: String,
    is_complete: bool,
}

#[derive(Deserialize, Serialize)]
struct TodoInput {
    content: String,
    is_complete: bool,
}

impl From<TodoInput> for Todo {
    fn from(input: TodoInput) -> Self {
        Todo {
            id: 0, // Will be set later
            content: input.content,
            is_complete: input.is_complete,
        }
    }
}

const DB_FILE: &str = "todos.json";

// Helper functions for file operations
fn load_todos() -> Vec<Todo> {
    if Path::new(DB_FILE).exists() {
        let data = fs::read_to_string(DB_FILE).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_todos(todos: &Vec<Todo>) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_json::to_string_pretty(todos)?;
    fs::write(DB_FILE, data)?;
    Ok(())
}

fn get_next_id(todos: &Vec<Todo>) -> i32 {
    todos.iter().map(|t| t.id).max().unwrap_or(0) + 1
}

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // GET /todos - List all todos
    app.get("/todos", |_req, res| async move {
        let todos = load_todos();
        res.ok().json(todos)
    });

    // GET /todo/:id - Get specific todo
    app.get("/todo/:id", |req, res| async move {
        let id = match req.params.get_int("id") {
            Ok(id) => id,
            Err(err) => {
                return res.bad_request().json(json!({
                    "error": err
                }));
            }
        };

        let todos = load_todos();
        match todos.iter().find(|todo| todo.id == id) {
            Some(todo) => res.ok().json(todo),
            None => res.not_found().json(json!({
                "error": "Todo not found"
            })),
        }
    });

    // POST /todos - Create new todo
    app.post("/todos", |req, res| async move {
        let todo_input = match req.json::<TodoInput>() {
            Ok(data) => data,
            Err(err) => {
                return res.bad_request().json(json!({
                    "error": format!("Invalid JSON: {}", err)
                }));
            }
        };

        let mut todos = load_todos();
        let mut new_todo = Todo::from(todo_input);
        new_todo.id = get_next_id(&todos);
        todos.push(new_todo.clone());

        match save_todos(&todos) {
            Ok(_) => res.status(201).json(json!({
                "message": "Todo created successfully",
                "data": new_todo
            })),
            Err(_) => res.internal_server_error().json(json!({
                "error": "Failed to save todo"
            })),
        }
    });

    // PUT /todo/:id - Update todo
    app.put("/todo/:id", |req, res| async move {
        let todo_input = match req.json::<TodoInput>() {
            Ok(data) => data,
            Err(e) => {
                return res.bad_request().json(json!({
                    "error": format!("Invalid JSON: {}", e)
                }));
            }
        };

        let id = match req.params.get_int("id") {
            Ok(id) => id,
            Err(err) => {
                return res.bad_request().json(json!({
                    "error": err,
                    "message": "Invalid ID format"
                }));
            }
        };

        let mut todos = load_todos();
        match todos.iter_mut().find(|t| t.id == id) {
            Some(existing_todo) => {
                // Keep the existing ID, only update content and is_complete
                existing_todo.content = todo_input.content;
                existing_todo.is_complete = todo_input.is_complete;

                match save_todos(&todos) {
                    Ok(_) => res.ok().json(json!({
                        "message": "Todo updated successfully"
                    })),
                    Err(_) => res.internal_server_error().json(json!({
                        "error": "Failed to save todo"
                    })),
                }
            }
            None => res.not_found().json(json!({
                "error": "Todo not found"
            })),
        }
    });

    // DELETE /todo/:id - Delete todo
    app.delete("/todo/:id", |req, res| async move {
        let id = match req.params.get_int("id") {
            Ok(id) => id,
            Err(err) => {
                return res.bad_request().json(json!({
                    "error": err,
                }));
            }
        };

        let mut todos = load_todos();
        let todo_index = todos.iter().position(|t| t.id == id);

        match todo_index {
            Some(index) => {
                let removed_todo = todos.remove(index);
                match save_todos(&todos) {
                    Ok(_) => res.ok().json(json!({
                        "message": "Todo deleted successfully",
                        "data": removed_todo
                    })),
                    Err(_) => res.internal_server_error().json(json!({
                        "error": "Failed to save changes"
                    })),
                }
            }
            None => res.not_found().json(json!({
                "error": "Todo not found"
            })),
        }
    });

    app.listen(3000, || {
        println!("🚀 Server running on http://localhost:3000");
    })
    .await;
}
