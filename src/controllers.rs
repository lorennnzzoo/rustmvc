use crate::ActionResult;
use crate::RequestContext;
use std::sync::Arc;
pub struct HomeController;
use crate::RenderModel;
use crate::Template;
#[derive(Template)]
#[template(path = "index.html")]
pub struct HomeModel {
    pub title: String,
    pub message: String,
    pub show_welcome: bool,
    pub user_name: String,
    pub products: Vec<Product>,
}

pub struct Product {
    pub name: String,
    pub price: f64,
}
impl RenderModel for HomeModel {
    fn render_html(&self) -> Result<String, askama::Error> {
        self.render()
    }
}
impl HomeController {
    pub fn index(_request_context: RequestContext) -> ActionResult {
        let model = HomeModel {
            title: "Rust MVC Askama Demo".to_string(),
            message: "This is a dynamic page with Rust logic inside the template.".to_string(),
            show_welcome: true,
            user_name: "Lorenzo".to_string(),
            products: vec![
                Product {
                    name: "Apple".to_string(),
                    price: 2.0,
                },
                Product {
                    name: "Banana".to_string(),
                    price: 1.5,
                },
            ],
        };

        ActionResult::View(Arc::new(model))
    }
}
