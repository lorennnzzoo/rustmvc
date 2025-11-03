use rustmvc::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::new();

    server.add_route("/products/show", ProductsController::show);
    server.add_route("/products/list", ProductsController::list);

    server.start("127.0.0.1:8080").await
}
