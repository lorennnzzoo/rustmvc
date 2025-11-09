
# RustMVC
A lightweight MVC web framework for Rust, built on top of **Actix Web** and **Askama** templates.
`rustmvc` helps you organize your Rust web applications using the familiar **Model–View–Controller** pattern while keeping it fast, simple, and extensible.

***

### Features
- Actix Web-based routing and async HTTP handling
- Askama templating support (for safe, fast server-side rendering)
- Built-in **middleware** system
- Simple **request context**
- Route-based **rules** like authorization, size limits, and role restrictions
- Extensible **actions** and **responses**

***

### Installation

```toml
[dependencies]
actix-web = "4.11.0"
askama = "0.14.0"
mime_guess = "2.0.5"
rustmvc = { path = "./rustmvc" } # adjust path based on your workspace
```

***

### Quick Start Example

Below is a minimal example that defines a small MVC app using `rustmvc`.

```rust
use rustmvc::{
    Server, ActionResult, RequestContext, HttpMethod, RouteRules, RenderModel, Template,
};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    message: String,
}

impl RenderModel for IndexTemplate {
    fn render_html(&self) -> Result<String, askama::Error> {
        self.render()
    }
}

fn home(ctx: RequestContext) -> ActionResult {
    println!("Received request for {}", ctx.path);
    ActionResult::View(std::sync::Arc::new(IndexTemplate {
        message: "Welcome to RustMVC!".into(),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::new();
    server.add_route("/", home, HttpMethod::GET, vec![]);
    server.start("127.0.0.1:8080").await
}
```

When you open **http://127.0.0.1:8080**, it will render the Askama template `index.html` with your message.

***

### Core Concepts

#### 1. RequestContext
Represents the incoming HTTP request.
You receive this as a parameter in every controller action.

```rust
pub struct RequestContext {
    pub params: HashMap<String, String>,
    pub headers: HeaderMap,
    pub path: String,
    pub body: Vec<u8>,
    pub method: HttpMethod,
    pub rules: Vec<RouteRules>,
    pub user: Option<User>,
}
```

You can access things like headers, query params, and body easily:

```rust
fn submit(ctx: RequestContext) -> ActionResult {
    if let Some(token) = ctx.headers.get("Authorization") {
        println!("Token: {:?}", token);
    }
    ActionResult::Ok("Form Submitted".to_string())
}
```

***

#### 2. ActionResult
Every controller returns an **ActionResult**, which defines the HTTP response.

```rust
pub enum ActionResult {
    Html(String),
    View(ArcRenderModel),
    Redirect(String),
    File(String),
    NotFound,
    PayloadTooLarge(String),
    UnAuthorized(String),
    Forbidden(String),
    Ok(String),
    BadRequest(String),
}
```

Examples:

```rust
ActionResult::Html("<h1>Hello World</h1>".to_string());
ActionResult::Redirect("/login".to_string());
ActionResult::File("logo.png".to_string());
```

***

#### 3. Server

Acts as the main entry point to your application.
It stores the list of registered **routes**, **middlewares**, and **authentication configuration**.

##### Create a new instance
```rust
let mut server = Server::new();
```

##### Add a route
Each route maps a **path** and **HTTP method** to an **action function**.

```rust
server.add_route("/users", list_users, HttpMethod::GET, vec![RouteRules::AllowAnonymous]);
server.add_route("/upload", upload_file, HttpMethod::POST, vec![RouteRules::RequestSizeLimit(1024 * 1024)]);
```

##### Start the server
```rust
server.start("127.0.0.1:8080").await?;
```

The server automatically matches routes, applies middlewares, and handles results.

***

#### 4. Middleware

Middlewares are executed **in order** before the controller action runs.
They can modify the request, log activity, or even return responses directly.

```rust
server.add_middleware(|ctx, next| {
    println!("Middleware: {}", ctx.path);
    let res = next(ctx);
    println!("After response");
    res
});
```

You can stack multiple middlewares for logging, authentication, etc.
For example, you could log timing or enforce a global header.

***

#### 5. RouteRules

Rules allow attaching security or validation behavior to individual routes.

```rust
pub enum RouteRules {
    Authorize,
    AllowAnonymous,
    Roles(Vec<String>),
    RequestSizeLimit(usize),
}
```

Usage examples:

```rust
// Public route
server.add_route("/", home, HttpMethod::GET, vec![RouteRules::AllowAnonymous]);

// Restricted by payload size
server.add_route(
    "/upload",
    upload_action,
    HttpMethod::POST,
    vec![RouteRules::RequestSizeLimit(1024 * 1024)], // 1 MB
);
```

***

#### 6. RenderModel Trait

Your view models (Askama templates) must implement the `RenderModel` trait.

```rust
pub trait RenderModel: Send + Sync {
    fn render_html(&self) -> Result<String, askama::Error>;
}
```

Example with Askama:

```rust
#[derive(Template)]
#[template(path = "user.html")]
struct UserTemplate {
    name: String,
}

impl RenderModel for UserTemplate {
    fn render_html(&self) -> Result<String, askama::Error> {
        self.render()
    }
}

fn show_user(_: RequestContext) -> ActionResult {
    ActionResult::View(std::sync::Arc::new(UserTemplate {
        name: "Lorenzo".to_string(),
    }))
}
```

***

#### 7. Authentication (Optional)

The server supports JWT-based authentication via an `AuthConfig` that can generate and validate tokens.
This part can be extended by enabling the commented-out `set_auth_config` and validation logic.

Example token generation:

```rust
let claims = Claims {
    sub: "user123".into(),
    roles: vec!["admin".into()],
};
let token = server.generate_token(claims, 3600);
```

***

#### 8. File Serving

Static files (like assets) are served from the `wwwroot` directory automatically when returned via `ActionResult::File`.

```rust
fn show_logo(_: RequestContext) -> ActionResult {
    ActionResult::File("images/logo.png".into())
}
```

***

### Example Middleware Chain Execution Flow

If you register:
```rust
server.add_middleware(logging_middleware);
server.add_middleware(auth_middleware);
```

For any request, execution will be:

```
logging_middleware -> auth_middleware -> route_action -> response
```

This composition pattern makes feature layering (e.g., auth, logging, request limits) simple and modular.

***

### Example Folder Structure

```
project/
│
├── src/
│   ├── main.rs
│   ├── controllers/
│   │   └── home.rs
│   └── rustmvc/
│       ├── mod.rs
│       └── authentication.rs
│
├── templates/
│   └── index.html
│
└── wwwroot/
    ├── css/
    ├── js/
    └── images/
```

***

### Summary

RustMVC is ideal for:
- Building lightweight, structured web servers in Rust
- Integrating Askama templates for server-rendered pages
- Managing middlewares, routing, and request contexts cleanly within Actix Web

If you already use **Actix Web** and want structured controllers, type-safe views, and rule-based routing, RustMVC provides a great foundation.
