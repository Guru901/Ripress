# App Object (App)

## Overview

`App` represents an application that can handle incoming HTTP requests and provide responses.

## Creating an App Object

```rust
use ripress::app::App;

let mut app = App::new();
```

## Handling Requests

### GET Requests

```rust

use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

let mut app = App::new();

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, World!")
}

app.get("/hello", handler);
```

### POST Requests

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

let mut app = App::new();

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, World!")
}

app.post("/hello", handler);
```

### PUT Requests

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

let mut app = App::new();

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, World!")
}

app.put("/hello", handler);
```

### DELETE Requests

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

let mut app = App::new();

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, World!")
}

app.delete("/hello", handler);
```

## Starting the Server

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

#[tokio::main]
fn main() {
    let mut app = App::new();
    app.listen("127.0.0.1:3000").await;
}
```

## Adding Routes with Parameters

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

let mut app = App::new();

app.get("/user/{id}", handler);

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.get_params("id").unwrap_or("unknown".to_string());
}
```
