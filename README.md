# Rust MVC Framework

A lightweight **Rust implementation of an MVC (Model-View-Controller) framework**, inspired by ASP.NET MVC. This framework allows you to define **controllers, actions, and views** in Rust and serve HTML pages dynamically. It integrates with **Askama** templates for compile-time HTML rendering.

---

## Features

* **Controllers & Actions**: Define Rust structs as controllers and functions as actions.
* **Dynamic Routes**: Map URL paths to controller actions easily.
* **ActionResult Enum**: Supports returning `Html`, `View`, `Redirect`, `File`, and `NotFound`.
* **Askama Templates**: Compile-time HTML rendering with Rust logic inside templates.
* **Static Files Support**: Serve files from `wwwroot`.
* **Lightweight HTTP Server**: Built on top of `actix-web`.

---

## Installation

Add this crate as a dependency in your project:

```toml
[dependencies]
rustmvc = { path = "../rustmvc" } # from GitHub
askama = { version = "0.12", features = ["macros"] }
actix-web = "4"
```

---

## Getting Started

### 1. Define a Model

```rust
use askama::Template;
use rustmvc::RenderModel;

#[derive(Template)]
#[template(path = "index.html")]
pub struct HomeModel {
    pub title: String,
    pub message: String,
}

impl RenderModel for HomeModel {
    fn render_html(&self) -> Result<String, askama::Error> {
        self.render()
    }
}
```

---

### 2. Define a Controller

```rust
use std::collections::HashMap;
use std::sync::Arc;
use rustmvc::{ActionResult, RenderModel};

pub struct HomeController;

impl HomeController {
    pub fn index(_params: HashMap<String, String>) -> ActionResult {
        let model = HomeModel {
            title: "Rust MVC Demo".to_string(),
            message: "Hello from Rust MVC!".to_string(),
        };
        ActionResult::View(Arc::new(model))
    }
}
```

---

### 3. Define Routes and Start the Server

```rust
use rustmvc::{Server, HomeController};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::new();

    server.add_route("/", HomeController::index);

    server.start("127.0.0.1:8080").await
}
```

---

### 4. Add a Template

Create a file `templates/index.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{{ title }}</title>
</head>
<body>
    <h1>{{ message }}</h1>
</body>
</html>
```

---

## Folder Structure

```
rustmvc/
├─ src/
│  ├─ controllers.rs
│  ├─ lib.rs
│  └─ main.rs
├─ templates/
│  └─ index.html
├─ wwwroot/
│  └─ static files here
├─ Cargo.toml
└─ README.md
```

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

---

## Notes

* Use `ActionResult::View` with **Arc-wrapped models** implementing `RenderModel` for dynamic rendering.
* Static files are served from the `wwwroot` folder.
* Askama templates are **precompiled** at build time for fast rendering.

---

This framework is a **work in progress** and serves as a learning project to replicate ASP.NET MVC concepts in Rust.
