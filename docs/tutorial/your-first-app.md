# Your First Ripress App - Todo API

## What We'll Build

A REST API for managing todos with these endpoints:

- GET /todos - List all todos
- POST /todos - Create a new todo
- GET /todo/:id - Get a specific todo
- PUT /todo/:id - Update a todo
- DELETE /todo/:id - Delete a todo

## Step 1: Project Setup

To create a new Ripress project, run the following command in your terminal:

```bash
cargo new my-ripress-app
cd my-ripress-app
# Add dependencies with correct features per crate
cargo add ripress
cargo add tokio --features macros,rt-multi-thread
cargo add serde --features derive
cargo add serde_json
```

### WebSocket Support (Optional)

If you want to add WebSocket support to your app, add the wynd dependency:

```bash
cargo add wynd
```

The "with-wynd" feature is enabled by default in Ripress, so WebSocket support will be available automatically.

## Step 2: Basic "Hello World"

Go to `src/main.rs` and replace the contents with the following:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", hello_world);

    app.listen(3000, || {
        println!("🚀 Server running on http://localhost:3000");
    })
    .await;
}

async fn hello_world(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(200).text("Hello, Ripress!")
}
```

Run your server:

```bash
cargo run
```

You should see:

```
🚀 Server running on http://localhost:3000
```

Visit `http://localhost:3000` in your browser or use curl:

```bash
curl http://localhost:3000
```

You should get:

```
Hello, Ripress!
```

## Step 3: Data Models and File Storage

Now let's create our data models and file-based storage. Replace your `src/main.rs` with:

```rust
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

    app.listen(3000, || {
        println!("🚀 Server running on http://localhost:3000");
    })
    .await;
}
```

## Step 4: List Todos (GET /todos)

Add the endpoint to list all todos:

```rust
// GET /todos - List all todos
app.get("/todos", |_req, res| async move {
    let todos = load_todos();
    res.ok().json(todos)
});
```

Test it:

```bash
curl http://localhost:3000/todos
```

Expected response (initially empty):

```json
[]
```

## Step 5: Create Todo (POST /todos)

Add the endpoint to create a new todo:

```rust
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
```

Test it:

```bash
curl -X POST http://localhost:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"content": "Learn Ripress", "is_complete": false}'
```

Expected response:

```json
{
  "message": "Todo created successfully",
  "data": {
    "id": 1,
    "content": "Learn Ripress",
    "is_complete": false
  }
}
```

## Step 6: Get Single Todo (GET /todo/:id)

Add the endpoint to get a specific todo:

```rust
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
```

Test it:

```bash
curl http://localhost:3000/todo/1
```

Expected response:

```json
{
  "id": 1,
  "content": "Learn Ripress",
  "is_complete": false
}
```

## Step 7: Update Todo (PUT /todo/:id)

Add the endpoint to update a todo:

```rust
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
```

Test it:

```bash
curl -X PUT http://localhost:3000/todo/1 \
  -H "Content-Type: application/json" \
  -d '{"content": "Learn Ripress - Updated", "is_complete": true}'
```

Expected response:

```json
{
  "message": "Todo updated successfully"
}
```

## Step 8: Delete Todo (DELETE /todo/:id)

Add the endpoint to delete a todo:

```rust
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
```

Test it:

```bash
curl -X DELETE http://localhost:3000/todo/1
```

Expected response:

```json
{
  "message": "Todo deleted successfully",
  "data": {
    "id": 1,
    "content": "Learn Ripress - Updated",
    "is_complete": true
  }
}
```

## Step 9: Error Handling & Validation

The code already includes comprehensive error handling:

- **JSON Parsing Errors**: Invalid request bodies return 400 Bad Request
- **Parameter Validation**: Invalid ID parameters return 400 Bad Request
- **Resource Not Found**: Missing todos return 404 Not Found
- **File System Errors**: Storage failures return 500 Internal Server Error

Key error handling patterns used:

- `match` statements for graceful error handling
- Proper HTTP status codes (400, 404, 500, 201)
- Descriptive error messages in JSON format

## Step 10: Testing Your API

Here's a complete test sequence:

```bash
# 1. List todos (should be empty initially)
curl http://localhost:3000/todos

# 2. Create a todo
curl -X POST http://localhost:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"content": "Learn Ripress", "is_complete": false}'

# 3. Create another todo
curl -X POST http://localhost:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"content": "Build an API", "is_complete": false}'

# 4. List all todos
curl http://localhost:3000/todos

# 5. Get a specific todo
curl http://localhost:3000/todo/1

# 6. Update a todo
curl -X PUT http://localhost:3000/todo/1 \
  -H "Content-Type: application/json" \
  -d '{"content": "Learn Ripress - Completed", "is_complete": true}'

# 7. Delete a todo
curl -X DELETE http://localhost:3000/todo/2

# 8. List todos to see changes
curl http://localhost:3000/todos
```

## Step 11: Adding WebSocket Support (Optional)

If you want to add real-time features to your Todo API, you can integrate WebSocket support. This allows clients to receive live updates when todos are created, updated, or deleted.

### Adding WebSocket Dependencies

First, add the wynd dependency to your `Cargo.toml`:

```bash
cargo add wynd
```

### Creating a WebSocket-Enhanced Todo API

Here's how to add WebSocket support to your Todo API:

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast;

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

#[derive(Serialize)]
struct TodoEvent {
    event_type: String,
    todo: Option<Todo>,
    message: String,
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
    let mut wynd = Wynd::new();

    // Create a broadcast channel for todo events
    let (tx, _rx) = broadcast::channel::<TodoEvent>(100);
    let tx = Arc::new(tx);

    // HTTP route for the todo management page
    app.get("/", |_, res| async move {
        res.ok().html(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Todo App with WebSocket</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 20px; }
                    .todo-item { margin: 10px 0; padding: 10px; border: 1px solid #ccc; }
                    .completed { background-color: #e8f5e8; }
                    #events { height: 200px; overflow-y: scroll; border: 1px solid #ccc; padding: 10px; margin-top: 20px; }
                    .event { margin: 5px 0; padding: 5px; background: #f0f0f0; }
                </style>
            </head>
            <body>
                <h1>Todo App with Real-time Updates</h1>
                <div id="todos"></div>
                <div>
                    <input type="text" id="newTodo" placeholder="New todo content">
                    <button onclick="addTodo()">Add Todo</button>
                </div>
                <h3>Real-time Events:</h3>
                <div id="events"></div>

                <script>
                    const ws = new WebSocket('ws://localhost:3000/ws');
                    const todosDiv = document.getElementById('todos');
                    const eventsDiv = document.getElementById('events');

                    ws.onmessage = function(event) {
                        const data = JSON.parse(event.data);
                        addEvent(data.message);

                        if (data.event_type === 'todo_updated') {
                            loadTodos(); // Refresh todo list
                        }
                    };

                    function addEvent(message) {
                        const div = document.createElement('div');
                        div.className = 'event';
                        div.textContent = new Date().toLocaleTimeString() + ': ' + message;
                        eventsDiv.appendChild(div);
                        eventsDiv.scrollTop = eventsDiv.scrollHeight;
                    }

                    async function loadTodos() {
                        const response = await fetch('/todos');
                        const todos = await response.json();
                        displayTodos(todos);
                    }

                    function displayTodos(todos) {
                        todosDiv.innerHTML = '';
                        todos.forEach(todo => {
                            const div = document.createElement('div');
                            div.className = 'todo-item' + (todo.is_complete ? ' completed' : '');
                            div.innerHTML = `
                                <input type="checkbox" ${todo.is_complete ? 'checked' : ''}
                                       onchange="toggleTodo(${todo.id}, this.checked)">
                                <span>${todo.content}</span>
                                <button onclick="deleteTodo(${todo.id})">Delete</button>
                            `;
                            todosDiv.appendChild(div);
                        });
                    }

                    async function addTodo() {
                        const input = document.getElementById('newTodo');
                        const content = input.value.trim();
                        if (!content) return;

                        const response = await fetch('/todos', {
                            method: 'POST',
                            headers: {'Content-Type': 'application/json'},
                            body: JSON.stringify({content, is_complete: false})
                        });

                        if (response.ok) {
                            input.value = '';
                            loadTodos();
                        }
                    }

                    async function toggleTodo(id, isComplete) {
                        const response = await fetch(`/todo/${id}`, {
                            method: 'PUT',
                            headers: {'Content-Type': 'application/json'},
                            body: JSON.stringify({content: 'Updated', is_complete: isComplete})
                        });

                        if (response.ok) {
                            loadTodos();
                        }
                    }

                    async function deleteTodo(id) {
                        const response = await fetch(`/todo/${id}`, {method: 'DELETE'});
                        if (response.ok) {
                            loadTodos();
                        }
                    }

                    // Load todos on page load
                    loadTodos();
                </script>
            </body>
            </html>
        "#)
    });

    // GET /todos - List all todos
    app.get("/todos", |_req, res| async move {
        let todos = load_todos();
        res.ok().json(todos)
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
            Ok(_) => {
                // Broadcast todo creation event
                let event = TodoEvent {
                    event_type: "todo_created".to_string(),
                    todo: Some(new_todo.clone()),
                    message: format!("Todo '{}' created", new_todo.content),
                };

                if let Err(e) = tx.send(event) {
                    eprintln!("Failed to broadcast todo creation: {}", e);
                }

                res.status(201).json(json!({
                    "message": "Todo created successfully",
                    "data": new_todo
                }))
            },
            Err(_) => {
                res.internal_server_error().json(json!({
                    "error": "Failed to save todo"
                }))
            },
        }
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
                let old_content = existing_todo.content.clone();
                existing_todo.content = todo_input.content;
                existing_todo.is_complete = todo_input.is_complete;

                match save_todos(&todos) {
                    Ok(_) => {
                        // Broadcast todo update event
                        let event = TodoEvent {
                            event_type: "todo_updated".to_string(),
                            todo: Some(existing_todo.clone()),
                            message: format!("Todo '{}' updated", old_content),
                        };

                        if let Err(e) = tx.send(event) {
                            eprintln!("Failed to broadcast todo update: {}", e);
                        }

                        res.ok().json(json!({
                            "message": "Todo updated successfully"
                        }))
                    },
                    Err(_) => {
                        res.internal_server_error().json(json!({
                            "error": "Failed to save todo"
                        }))
                    },
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
                    Ok(_) => {
                        // Broadcast todo deletion event
                        let event = TodoEvent {
                            event_type: "todo_deleted".to_string(),
                            todo: Some(removed_todo.clone()),
                            message: format!("Todo '{}' deleted", removed_todo.content),
                        };

                        if let Err(e) = tx.send(event) {
                            eprintln!("Failed to broadcast todo deletion: {}", e);
                        }

                        res.ok().json(json!({
                            "message": "Todo deleted successfully",
                            "data": removed_todo
                        }))
                    },
                    Err(_) => {
                        res.internal_server_error().json(json!({
                            "error": "Failed to save changes"
                        }))
                    },
                }
            }
            None => res.not_found().json(json!({
                "error": "Todo not found"
            })),
        }
    });

    // WebSocket connection handler
    wynd.on_connection(|conn| async move {
        let tx = tx.clone();

        println!("New WebSocket client connected");

        // Handle incoming messages
        conn.on_text(|event, _| async move {
            println!("WebSocket message: {}", event.data);

            // Send welcome message
            let welcome = TodoEvent {
                event_type: "welcome".to_string(),
                todo: None,
                message: "Connected to Todo WebSocket server".to_string(),
            };

            if let Err(e) = event.handle.send_text(&serde_json::to_string(&welcome).unwrap()).await {
                eprintln!("Failed to send welcome message: {}", e);
            }
        });

        // Handle connection close
        conn.on_close(|event, _| async move {
            println!("WebSocket client disconnected: {:?}", event.code);
        });

        // Handle errors
        conn.on_error(|event, _| async move {
            eprintln!("WebSocket error: {:?}", event.error);
        });
    });

    // Mount WebSocket at /ws path
    app.use_wynd("/ws", wynd.handler());

    app.listen(3000, || {
        println!("🚀 Todo server with WebSocket running on http://localhost:3000");
        println!("🔌 WebSocket available at ws://localhost:3000/ws");
    })
    .await;
}
```

### Testing the WebSocket-Enhanced Todo API

1. **Start the server**: `cargo run`
2. **Open the web interface**: Visit `http://localhost:3000`
3. **Open multiple browser tabs**: Each tab will receive real-time updates
4. **Create, update, or delete todos**: Watch the real-time events in each tab
5. **Test WebSocket connection**: Use browser dev tools to see WebSocket messages

### Key Features Added

- **Real-time updates**: All connected clients receive live updates
- **Event broadcasting**: Todo changes are broadcast to all WebSocket clients
- **Web interface**: Built-in HTML interface for testing
- **Event logging**: Real-time event feed shows all todo operations

## Complete Code

Your final `src/main.rs` should contain all the provided code, which creates a fully functional Todo API with persistent file storage.

## Key Features Implemented

✅ **CRUD Operations**: Create, Read, Update, Delete todos  
✅ **Persistent Storage**: Data saved to `todos.json` file  
✅ **Error Handling**: Comprehensive error responses  
✅ **JSON API**: All endpoints return JSON responses  
✅ **ID Management**: Automatic ID generation and validation  
✅ **HTTP Status Codes**: Proper REST API status codes

## Next Steps

Now that you have a working Todo API, you can extend it with:

1. **Database Integration**: Replace file storage with SQLite, PostgreSQL, or MongoDB
2. **Authentication**: Add user authentication and authorization
3. **Validation**: Add more robust input validation using a validation library
4. **Middleware**: Add logging, CORS, rate limiting
5. **Frontend**: Build a web interface to interact with your API
6. **Testing**: Add unit and integration tests
7. **Deployment**: Deploy to a cloud platform like Heroku or AWS
8. **Documentation**: Generate API documentation with OpenAPI/Swagger

Congratulations! You've built your first REST API with Ripress. The framework's simple closure-based routing makes it easy to build fast, reliable web services in Rust.

[Completed code](./your_first_app.rs)
