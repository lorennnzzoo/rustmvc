//! # RustMVC
//!
//! A lightweight MVC framework for Rust, built on top of Actix Web and Askama templates.
//! Provides routing, middlewares, request context, and response handling.

use crate::authentication::{AuthConfig, Claims};
use actix_web::http::header::HeaderMap;
use actix_web::http::Method;
use actix_web::web::Bytes;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
pub use askama;
pub use askama::Template;
use std::collections::HashMap;
use std::sync::Arc;
pub mod authentication;

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
    ///Http Method
    pub method: HttpMethod,
    /// Rules that are set for the path
    pub rules: Vec<RouteRules>,
    /// User context
    pub user: Option<User>,
}
///User context
#[derive(Clone)]
pub struct User {
    pub name: String,
    pub roles: Vec<String>,
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
    /// Pay Load Too Large
    PayloadTooLarge(String),
    /// UnAuthorized
    UnAuthorized(String),
    /// Forbidden
    Forbidden(String),
    /// Ok
    Ok(String),
    /// BadRequest
    BadRequest(String),
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
///Rules for a route to pass before proceeding to action
#[derive(Clone, PartialEq, Eq)]
pub enum RouteRules {
    Authorize,
    AllowAnonymous,
    Roles(Vec<String>),
    RequestSizeLimit(usize),
}
/// Http Methods
#[derive(Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
    TRACE,
    CONNECT,
    NotSupported,
}

/// Represents a route in the server
#[derive(Clone)]
pub struct Route {
    /// The path to match (e.g., `/about`)
    pub path: String,
    /// The action to execute when the route is matched
    pub action: ActionFn,
    /// Route Rules
    pub rules: Vec<RouteRules>,
    /// Http Method
    pub method: HttpMethod,
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
    /// Auth Config to set secret key
    auth_config: Option<Arc<AuthConfig>>,
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
            auth_config: None,
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
                ActionResult::PayloadTooLarge(content) => println!("Response: {:?}", content),
                ActionResult::Forbidden(content) => println!("Response: {:?}", content),
                ActionResult::UnAuthorized(content) => println!("Response: {:?}", content),
                ActionResult::Ok(content) => println!("Response: {:?}", content),
                ActionResult::BadRequest(content) => println!("Response: {:?}", content),
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
    /// Set Auth Config
    // pub fn set_auth_config(&mut self, config: AuthConfig) {
    //     self.auth_config = Some(Arc::new(config));
    // }
    // pub fn get_auth_config(&self) -> Option<Arc<AuthConfig>> {
    //     self.auth_config.clone()
    // }
    /// Helper to generate token from user-provided claims
    pub fn generate_token(&self, claims: Claims, expires_in_secs: i64) -> Option<String> {
        self.auth_config
            .as_ref()
            .map(|cfg| cfg.generate_token(&claims.sub, claims.roles.clone(), expires_in_secs))
    }
    /// Register a route with the server
    ///
    /// # Example
    /// ```rust
    /// server.add_route("/", HomeController::index);
    /// ```
    pub fn add_route<F>(
        &mut self,
        path: &str,
        action: F,
        method: HttpMethod,
        rules: Vec<RouteRules>,
    ) where
        F: Fn(RequestContext) -> ActionResult + Send + Sync + 'static,
    {
        self.routes.push(Route {
            path: path.to_string(),
            action: Arc::new(action),
            method,
            rules,
        });
    }
    /// Internal function to handle an incoming request
    fn handle_request(&self, ctx: RequestContext) -> ActionResult {
        let route = match self
            .routes
            .iter()
            .find(|r| r.path == ctx.path && r.method == ctx.method)
        {
            Some(r) => r,
            None => return ActionResult::NotFound,
        };

        // Check NotSupported Routes
        if route.method == HttpMethod::NotSupported {
            return ActionResult::NotFound;
        }
        // Check if a route with Authorize Rule has valid token passed
        // if route.rules.contains(&RouteRules::Authorize) {
        //     let auth = match &self.auth_config {
        //         Some(a) => a,
        //         None => return ActionResult::UnAuthorized("No auth config".into()),
        //     };

        //     if let Some(auth_header) = ctx.headers.get("Authorization") {
        //         let token = auth_header.to_str().unwrap_or("").replace("Bearer ", "");
        //         if auth.validate_token(&token).is_err() {
        //             return ActionResult::UnAuthorized("Invalid token".into());
        //         }
        //     } else {
        //         return ActionResult::UnAuthorized("Missing token".into());
        //     }
        // }

        // Check route rules first
        for rule in route.rules.clone() {
            match rule {
                RouteRules::RequestSizeLimit(limit) => {
                    if ctx.body.len() > limit {
                        return ActionResult::PayloadTooLarge(format!(
                            "Request to route '{}' exceeded the allowed size: {} bytes",
                            route.path, limit
                        ));
                    }
                }
                _ => (),
            }
        }

        // Compose middlewares
        let mut next: ActionFn = route.action.clone();
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

                        let mapped_methods = match req.method() {
                            &Method::GET => HttpMethod::GET,
                            &Method::POST => HttpMethod::POST,
                            &Method::PUT => HttpMethod::PUT,
                            &Method::DELETE => HttpMethod::DELETE,
                            &Method::PATCH => HttpMethod::PATCH,
                            &Method::CONNECT => HttpMethod::CONNECT,
                            &Method::OPTIONS => HttpMethod::OPTIONS,
                            &Method::HEAD => HttpMethod::HEAD,
                            &Method::TRACE => HttpMethod::TRACE,
                            _ => HttpMethod::NotSupported,
                        };

                        let route_rules = match srv.routes.iter().find(|r| {
                            r.path == req.path().to_string() && r.method == mapped_methods
                        }) {
                            Some(r) => r.rules.clone(),
                            None => Vec::new(),
                        };

                        let ctx = RequestContext {
                            path: req.path().to_string(),
                            headers: req.headers().clone(),
                            params,
                            body: body.to_vec(),
                            method: mapped_methods,
                            rules: route_rules,
                            user: None,
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
                            ActionResult::Ok(content) => HttpResponse::Ok().body(content),
                            ActionResult::BadRequest(content) => {
                                HttpResponse::BadRequest().body(content)
                            }
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
                            ActionResult::PayloadTooLarge(body) => {
                                HttpResponse::PayloadTooLarge().body(body)
                            }

                            ActionResult::Forbidden(body) => HttpResponse::Forbidden().body(body),
                            ActionResult::UnAuthorized(body) => {
                                HttpResponse::Unauthorized().body(body)
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
