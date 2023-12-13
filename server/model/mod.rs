mod document;
mod document_config;
mod collection;
mod engine;

use std::collections::HashMap;

pub struct Request {
    header: HashMap<String, String>,
    body: Vec<u8>
}
