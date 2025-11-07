//! # RustMVC
//!
//! A lightweight MVC framework for Rust, built on top of Actix Web and Askama templates.
//! Provides routing, middlewares, request context, and response handling.

use actix_web::http::header::HeaderMap;
use actix_web::web::Bytes;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
pub use askama;
pub use askama::Template;
use std::collections::HashMap;
use std::sync::Arc;

/// Shared pointer to a type implementing the `RenderModel` trait.
pub type ArcRenderModel = Arc<dyn RenderModel>;

/// Contains information about an incoming HTTP request.
#[derive(Clone)]
pub struct RequestContext {
    /// Query parameters from the URL (e.g., `/path?foo=bar` -> `{"foo": "bar"}`)
    pub params: HashMap<String, String>,
    /// HTTP headers of the request
    pub headers: HeaderMap,
    /// The path of the request (e.g., `/about`)
    pub path: String,
    /// Request body bytes (useful for POST/PUT requests)
    pub body: Vec<u8>,
}

/// Represents the possible responses an action can return.
#[derive(Clone)]
pub enum ActionResult {
    /// HTML content as a raw string
    Html(String),
    /// Render a model implementing `RenderModel` (e.g., Askama templates)
    View(ArcRenderModel),
    /// Redirect to another URL
    Redirect(String),
    /// Return a static file (served from `wwwroot`)
    File(String),
    /// 404 Not Found
    NotFound,
}
/// Trait implemented by models that can render themselves to HTML.
pub trait RenderModel: Send + Sync {
    /// Render the model into an HTML string
    fn render_html(&self) -> Result<String, askama::Error>;
}
/// Type of an action function (controller handler)
pub type ActionFn = Arc<dyn Fn(RequestContext) -> ActionResult + Send + Sync + 'static>;

/// Type of a middleware function
pub type MiddlewareFn =
    Arc<dyn Fn(RequestContext, ActionFn) -> ActionResult + Send + Sync + 'static>;

/// Represents a route in the server
pub struct Route {
    /// The path to match (e.g., `/about`)
    pub path: String,
    /// The action to execute when the route is matched
    pub action: ActionFn,
}
/// The main server struct of RustMVC.
///
/// Holds all the registered routes and middlewares.
/// Users create a `Server`, register routes and middlewares, and then start it.
pub struct Server {
    /// A vector of registered routes.
    /// Each route has a path and an action function.
    routes: Vec<Route>,
    /// A vector of middlewares.
    /// Middlewares are functions that wrap around route execution,
    /// allowing logging, authentication, request modification, etc.
    middlewares: Vec<MiddlewareFn>,
}

impl Server {
    /// Creates a new instance of the server with default logging middleware
    ///
    /// Example:
    /// ```rust
    /// let server = rustmvc::Server::new();
    /// ```
    pub fn new() -> Self {
        let mut server = Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
        };
        // Default logging middleware
        server.add_middleware(|ctx, next| {
            println!("--- Incoming Request ---");
            println!("Path: {}", ctx.path);
            println!("Query Params: {:?}", ctx.params);
            println!("Headers:");
            for (key, value) in ctx.headers.iter() {
                println!("  {}: {:?}", key, value);
            }
            println!("------------------------");

            let result = next(ctx.clone());

            match &result {
                ActionResult::Html(_) => println!("Response: Html"),
                ActionResult::View(_) => println!("Response: View"),
                ActionResult::Redirect(url) => println!("Response: Redirect to {}", url),
                ActionResult::File(path) => println!("Response: File {}", path),
                ActionResult::NotFound => println!("Response: NotFound"),
            }
            println!("--- End of Request ---\n");

            result
        });
        server
    }
    /// Add a middleware to the server
    ///
    /// Middlewares are executed in the order they are added.
    ///
    /// # Example
    /// ```rust
    /// server.add_middleware(|ctx, next| {
    ///     println!("Logging request: {}", ctx.path);
    ///     next(ctx)
    /// });
    /// ```
    pub fn add_middleware<F>(&mut self, mw: F)
    where
        F: Fn(RequestContext, ActionFn) -> ActionResult + Send + Sync + 'static,
    {
        self.middlewares.push(Arc::new(mw));
    }
    /// Register a route with the server
    ///
    /// # Example
    /// ```rust
    /// server.add_route("/", HomeController::index);
    /// ```
    pub fn add_route<F>(&mut self, path: &str, action: F)
    where
        F: Fn(RequestContext) -> ActionResult + Send + Sync + 'static,
    {
        self.routes.push(Route {
            path: path.to_string(),
            action: Arc::new(action),
        });
    }
    /// Internal function to handle an incoming request
    fn handle_request(&self, ctx: RequestContext) -> ActionResult {
        let route = match self.routes.iter().find(|r| r.path == ctx.path) {
            Some(r) => r.action.clone(),
            None => return ActionResult::NotFound,
        };

        // Compose middlewares
        let mut next: ActionFn = route;
        for mw in self.middlewares.iter().rev() {
            let current_next = next.clone();
            let mw_clone = mw.clone();
            next = Arc::new(move |ctx: RequestContext| -> ActionResult {
                mw_clone(ctx, current_next.clone())
            }) as ActionFn;
        }

        next(ctx)
    }
    /// Start the server asynchronously
    ///
    /// # Example
    /// ```rust
    /// actix_web::rt::System::new().block_on(async {
    ///     server.start("127.0.0.1:8080").await.unwrap();
    /// });
    /// ```
    pub async fn start(self, addr: &str) -> std::io::Result<()> {
        println!("Server listening at http://{}", addr);
        let shared_routes = web::Data::new(self);

        HttpServer::new(move || {
            App::new()
                .app_data(shared_routes.clone())
                .default_service(web::to(
                    |req: HttpRequest, body: Bytes, srv: web::Data<Server>| {
                        let mut params = HashMap::new();
                        for (key, value) in req
                            .query_string()
                            .split('&')
                            .filter(|s| !s.is_empty())
                            .map(|pair| {
                                let mut kv = pair.splitn(2, '=');
                                (kv.next().unwrap_or(""), kv.next().unwrap_or(""))
                            })
                        {
                            params.insert(key.to_string(), value.to_string());
                        }

                        let ctx = RequestContext {
                            path: req.path().to_string(),
                            headers: req.headers().clone(),
                            params,
                            body: body.to_vec(),
                        };

                        let result = srv.handle_request(ctx);

                        let body = match result {
                            ActionResult::Html(s) => {
                                HttpResponse::Ok().content_type("text/html").body(s)
                            }
                            ActionResult::View(renderer_arc) => match renderer_arc.render_html() {
                                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                                Err(e) => {
                                    eprintln!("Askama Rendering Error: {}", e);
                                    HttpResponse::InternalServerError()
                                        .body(format!("<h1>Template Rendering Error: {}</h1>", e))
                                }
                            },

                            ActionResult::Redirect(url) => HttpResponse::Found()
                                .append_header(("Location", url))
                                .finish(),
                            ActionResult::File(path) => {
                                let path = format!("wwwroot/{}", path);
                                match std::fs::read(&path) {
                                    Ok(bytes) => {
                                        let content_type =
                                            mime_guess::from_path(&path).first_or_octet_stream();
                                        HttpResponse::Ok()
                                            .content_type(content_type.as_ref())
                                            .body(bytes)
                                    }
                                    Err(_) => {
                                        HttpResponse::NotFound().body("<h1>404 Not Found</h1>")
                                    }
                                }
                            }

                            ActionResult::NotFound => {
                                HttpResponse::NotFound().body("<h1>404 Not Found</h1>")
                            }
                        };

                        async move { body }
                    },
                ))
        })
        .bind(addr)?
        .run()
        .await
    }
}
