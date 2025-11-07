use actix_web::http::header::HeaderMap;
use actix_web::web::Bytes;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
pub use askama;
pub use askama::Template;
use std::collections::HashMap;
use std::sync::Arc;

pub type ArcRenderModel = Arc<dyn RenderModel>;

#[derive(Clone)]
pub struct RequestContext {
    pub params: HashMap<String, String>,
    pub headers: HeaderMap,
    pub path: String,
    pub body: Vec<u8>,
}

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
pub type ActionFn = Arc<dyn Fn(RequestContext) -> ActionResult + Send + Sync + 'static>;

pub type MiddlewareFn =
    Arc<dyn Fn(RequestContext, ActionFn) -> ActionResult + Send + Sync + 'static>;

pub struct Route {
    pub path: String,
    pub action: ActionFn,
}

pub struct Server {
    routes: Vec<Route>,
    middlewares: Vec<MiddlewareFn>,
}

impl Server {
    pub fn new() -> Self {
        let mut server = Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
        };
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
    pub fn add_middleware<F>(&mut self, mw: F)
    where
        F: Fn(RequestContext, ActionFn) -> ActionResult + Send + Sync + 'static,
    {
        self.middlewares.push(Arc::new(mw));
    }
    pub fn add_route<F>(&mut self, path: &str, action: F)
    where
        F: Fn(RequestContext) -> ActionResult + Send + Sync + 'static,
    {
        self.routes.push(Route {
            path: path.to_string(),
            action: Arc::new(action),
        });
    }

    fn handle_request(&self, ctx: RequestContext) -> ActionResult {
        let route = match self.routes.iter().find(|r| r.path == ctx.path) {
            Some(r) => r.action.clone(),
            None => return ActionResult::NotFound,
        };

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
