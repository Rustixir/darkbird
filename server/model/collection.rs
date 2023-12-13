use crate::{Options, Storage};
use crate::server::model::document::Document;
use crate::server::model::document_config::DocumentConfig;

pub struct Collection {
    name: String,
    storage: Storage<String, Document>,
    config: Option<DocumentConfig>,
}

impl Collection {
    pub async fn new(name: String, storage_config: Option<Options>, document_config: Option<DocumentConfig>) -> Result<Self, String> {
        let result = Storage::open(Options::new(
            storage_config.path,
            storage_config.storage_name,
            storage_config.total_page_size,
            storage_config.storage_type,
            storage_config.off_reporter,
        )).await;

        match result {
            Ok(storage) => {
                return Ok(Collection {
                    name,
                    storage,
                    config: document_config,
                })
            }
            Err(e) => Err(e)
        }
    }

    pub async fn insert(&self, key: String, doc: Document) -> Result<(), SessionResult> {
        self.storage.insert()
    }
}

