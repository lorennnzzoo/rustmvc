use crate::ActionResult;
use std::collections::HashMap;
pub struct HomeController;

impl HomeController {
    pub fn index(_params: HashMap<String, String>) -> ActionResult {
        ActionResult::View("index".to_string())
    }
}
