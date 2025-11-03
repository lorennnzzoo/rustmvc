use crate::ActionResult;
use std::collections::HashMap;
pub struct ProductsController;

impl ProductsController {
    pub fn show(params: HashMap<String, String>) -> ActionResult {
        let id = params.get("id");

        match id {
            Some(id_value) => ActionResult::Html(format!("<h1>Product {:?}</h1>", id_value)),
            None => ActionResult::Html("<h1>Invalid param</h1>".to_string()),
        }
    }

    pub fn list(_params: HashMap<String, String>) -> ActionResult {
        ActionResult::Html("<h1>All Products</h1>".to_string())
    }
}
