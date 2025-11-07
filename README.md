# RustMVC

A lightweight MVC (Model-View-Controller) framework for Rust, built on top of Actix Web and Askama templates. RustMVC provides a simple, intuitive API for building web applications with routing, middleware support, and template rendering.

## Features

- **Simple Routing**: Register routes with path patterns and controller actions
- **Middleware Support**: Add custom middleware for logging, authentication, and request processing
- **Template Rendering**: Built-in support for Askama templates
- **Request Context**: Easy access to query parameters, headers, and request body
- **Multiple Response Types**: HTML, views, redirects, static files, and 404 handling
- **Built-in Logging**: Default middleware for request/response logging

## Installation

Add RustMVC to your `Cargo.toml`:

```toml
[dependencies]
rustmvc = "0.1.0"
actix-web = "4.11.0"
askama = "0.14.0"
```

## Quick Start

Here's a simple example to get you started:

```rust
use rustmvc::{Server, RequestContext, ActionResult, Template};
use std::sync::Arc;

// Define a view model with Askama template
#[derive(Template)]
#[template(path = "home.html")]
struct HomeView {
    title: String,
}

impl rustmvc::RenderModel for HomeView {
    fn render_html(&self) -> Result<String, askama::Error> {
        self.render()
    }
}

// Controller action
fn index(_ctx: RequestContext) -> ActionResult {
    let view = HomeView {
        title: "Welcome to RustMVC".to_string(),
    };
    ActionResult::View(Arc::new(view))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::new();

    // Register routes
    server.add_route("/", index);

    // Start the server
    server.start("127.0.0.1:8080").await
}
```

## Core Concepts

### Server

The `Server` struct is the main entry point. Create a new server instance, register routes and middleware, then start it:

```rust
let mut server = Server::new();
server.add_route("/about", about_controller);
server.start("127.0.0.1:8080").await?;
```

### Routes

Routes map URL paths to controller actions:

```rust
server.add_route("/users", list_users);
server.add_route("/users/profile", user_profile);
```

### Request Context

The `RequestContext` provides access to request information:

```rust
fn my_action(ctx: RequestContext) -> ActionResult {
    // Access query parameters
    let id = ctx.params.get("id");

    // Access headers
    let user_agent = ctx.headers.get("user-agent");

    // Access request path
    println!("Path: {}", ctx.path);

    // Access request body
    let body_data = &ctx.body;

    ActionResult::Html("<h1>Hello</h1>".to_string())
}
```

### Action Results

Controllers return `ActionResult` which can be one of several types:

```rust
// Return HTML directly
ActionResult::Html("<h1>Hello World</h1>".to_string())

// Render a view/template
ActionResult::View(Arc::new(my_view))

// Redirect to another URL
ActionResult::Redirect("/login".to_string())

// Serve a static file from wwwroot/
ActionResult::File("style.css".to_string())

// Return 404
ActionResult::NotFound
```

### Middleware

Add middleware to intercept and process requests:

```rust
// Authentication middleware
server.add_middleware(|ctx, next| {
    if ctx.params.get("auth_token").is_some() {
        next(ctx)
    } else {
        ActionResult::Redirect("/login".to_string())
    }
});

// Custom logging middleware
server.add_middleware(|ctx, next| {
    let start = std::time::Instant::now();
    let result = next(ctx);
    let duration = start.elapsed();
    println!("Request processed in {:?}", duration);
    result
});
```

Middlewares are executed in the order they are added.

### Templates with Askama

Create templates in the `templates/` directory:

```html
<!-- templates/home.html -->
<!DOCTYPE html>
<html>
<head>
    <title>{{ title }}</title>
</head>
<body>
    <h1>{{ title }}</h1>
</body>
</html>
```

Define a view model:

```rust
#[derive(Template)]
#[template(path = "home.html")]
struct HomeView {
    title: String,
}

impl rustmvc::RenderModel for HomeView {
    fn render_html(&self) -> Result<String, askama::Error> {
        self.render()
    }
}
```

Use it in your controller:

```rust
fn home(_ctx: RequestContext) -> ActionResult {
    ActionResult::View(Arc::new(HomeView {
        title: "My Website".to_string(),
    }))
}
```

### Static Files

Place static files in the `wwwroot/` directory. They can be served using `ActionResult::File`:

```rust
fn serve_css(_ctx: RequestContext) -> ActionResult {
    ActionResult::File("styles/main.css".to_string())
}
```

## Project Structure

```
my-app/
├── Cargo.toml
├── src/
│   └── main.rs
├── templates/
│   └── home.html
└── wwwroot/
    ├── css/
    │   └── style.css
    └── js/
        └── app.js
```

## Example Application

```rust
use rustmvc::{Server, RequestContext, ActionResult, Template};
use std::sync::Arc;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexView {
    message: String,
}

impl rustmvc::RenderModel for IndexView {
    fn render_html(&self) -> Result<String, askama::Error> {
        self.render()
    }
}

fn index(_ctx: RequestContext) -> ActionResult {
    ActionResult::View(Arc::new(IndexView {
        message: "Welcome!".to_string(),
    }))
}

fn about(_ctx: RequestContext) -> ActionResult {
    ActionResult::Html("<h1>About Us</h1>".to_string())
}

fn api_data(ctx: RequestContext) -> ActionResult {
    let name = ctx.params.get("name").cloned().unwrap_or_default();
    ActionResult::Html(format!("<p>Hello, {}</p>", name))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::new();

    server.add_route("/", index);
    server.add_route("/about", about);
    server.add_route("/api/greet", api_data);

    server.start("127.0.0.1:8080").await
}
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
