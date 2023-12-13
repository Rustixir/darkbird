use crate::Options;
use crate::server::model::collection::Collection;
use crate::server::model::document::Document;
use crate::server::model::document_config::DocumentConfig;
use crate::storage_redis::RedisStorage;

pub type Cache = RedisStorage<String, Document>;


pub enum Engine {
    DarkBird(Collection),
    Cache(Cache)
}

impl Engine {

    pub async fn new(name: String, storage_config: Option<Options>, document_config: Option<DocumentConfig>) -> Result<Self, String> {
        match Collection::new(name, storage_config, document_config).await {
            Ok(collection) => Ok(Engine::DarkBird(collection)),
            Err(e) => Err(e)
        }
    }

    pub fn insert(&self) -> Result<(), ()> {
        match self {
            Engine::DarkBird(collection) => {
                collection.insert()
            }
            Engine::Cache(cache) => {

            }
        }
    }
}