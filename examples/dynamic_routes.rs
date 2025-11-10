use crate::Server;
use rustmvc::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::new();

    server.get(
        "/say_hello/{name}",
        |ctx| {
            if let Some(name) = ctx.path_params.get("name") {
                return ActionResult::Ok(format!("Hello {}", name));
            }
            return ActionResult::Ok(format!("Hello anonymous"));
        },
        vec![],
    );

    server.start("127.0.0.1:8080").await
}
