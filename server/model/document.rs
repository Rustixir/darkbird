use std::sync::Arc;

use serde_json::Value;

use crate::document::{Document as Doc, FullText, Indexer, MaterializedView, Range, RangeField, Tags};
use crate::server::model::document_config::DocumentConfig;

pub struct Document {
    value: Value,
    config: Arc<DocumentConfig>,
}

impl Document {
    pub fn new(value: Value, config: Arc<DocumentConfig>) -> Document {
        Document {
            value,
            config,
        }
    }
}


impl Doc for Document {}

impl Indexer for Document {
    fn extract(&self) -> Vec<String> {
        if let Value::Object(doc) = &self.value {
            return
                self.config.index
                    .iter()
                    .filter_map(|index_field| doc.get(index_field))
                    .map(|val| val.to_string())
                    .collect::<Vec<String>>();
        }
        vec![]
    }
}

impl Tags for Document {
    fn get_tags(&self) -> Vec<String> {
        if let Value::Object(doc) = &self.value {
            return self.config.tags
                    .iter()
                    .filter_map(|index_field| doc.get(index_field))
                    .map(|val| val.to_string())
                    .collect::<Vec<String>>();
        }
        vec![]
    }
}

impl Range for Document {
    fn get_fields(&self) -> Vec<RangeField> {
        if let Value::Object(doc) = &self.value {
            return self.config.range
                    .iter()
                    .filter_map(|index_field| doc.get_key_value(index_field))
                    .map(|(field,val)| RangeField{ name: field.to_string(), value: val.to_string() })
                    .collect::<Vec<RangeField>>()
        }
        vec![]
    }
}

impl FullText for Document {
    fn get_content(&self) -> Option<String> {
        if let Value::Object(doc) = &self.value {
            return doc.get(&self.config.content)
                      .map(|v| v.to_string())
        }
        None
    }
}

// unimplemented for vsn 0.0.1
impl MaterializedView for Document {
    fn filter(&self) -> Option<String> {
        None
    }
}