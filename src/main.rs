use rustmvc::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::new();

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

    server.add_route("/", HomeController::index);
    server.start("127.0.0.1:8080").await
}
