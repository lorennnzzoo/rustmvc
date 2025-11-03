use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use std::collections::HashMap;
use std::sync::Arc;
mod controllers;
pub use askama;
pub use askama::Template;
pub use controllers::*;
// pub use serde::Serialize;
pub type ArcRenderModel = Arc<dyn RenderModel>;

#[derive(Clone)]
pub enum ActionResult {
    Html(String),
    View(ArcRenderModel),
    Redirect(String),
    File(String),
    NotFound,
}
pub trait RenderModel: Send + Sync {
    fn render_html(&self) -> Result<String, askama::Error>;
}
pub type ActionFn = Arc<dyn Fn(HashMap<String, String>) -> ActionResult + Send + Sync>;

pub struct Route {
    pub path: String,
    pub action: ActionFn,
}

pub struct Server {
    routes: Vec<Route>,
}

impl Server {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn add_route<F>(&mut self, path: &str, action: F)
    where
        F: Fn(HashMap<String, String>) -> ActionResult + Send + Sync + 'static,
    {
        self.routes.push(Route {
            path: path.to_string(),
            action: Arc::new(action),
        });
    }

    fn handle_request(&self, path: &str, params: HashMap<String, String>) -> ActionResult {
        for route in &self.routes {
            if route.path == path {
                return (route.action)(params);
            }
        }
        ActionResult::NotFound
    }

    pub async fn start(self, addr: &str) -> std::io::Result<()> {
        println!("Server listening at http://{}", addr);
        let shared_routes = web::Data::new(self);

        HttpServer::new(move || {
            App::new()
                .app_data(shared_routes.clone())
                .default_service(web::to(|req: HttpRequest, srv: web::Data<Server>| {
                    let mut params = HashMap::new();
                    for (key, value) in
                        req.query_string()
                            .split('&')
                            .filter(|s| !s.is_empty())
                            .map(|pair| {
                                let mut kv = pair.splitn(2, '=');
                                (kv.next().unwrap_or(""), kv.next().unwrap_or(""))
                            })
                    {
                        params.insert(key.to_string(), value.to_string());
                    }

                    let path = req.path();
                    let result = srv.handle_request(path, params);

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
                                Err(_) => HttpResponse::NotFound().body("<h1>404 Not Found</h1>"),
                            }
                        }

                        ActionResult::NotFound => {
                            HttpResponse::NotFound().body("<h1>404 Not Found</h1>")
                        }
                    };

                    async move { body }
                }))
        })
        .bind(addr)?
        .run()
        .await
    }
}
