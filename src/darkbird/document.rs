
pub trait Document: Indexer + Tags + Range + MaterializedView + FullText {}


// used for exracting fields for hash index
pub trait Indexer {
    fn extract(&self) -> Vec<String>;
}


// use for indexing group of documents by tag
pub trait Tags {
    fn get_tags(&self) -> Vec<String>;
}


// used for range index
pub trait Range {
    fn get_fields(&self) -> Vec<RangeField>;
}


// used for materialized view 
pub trait MaterializedView {
    fn filter(&self) -> Option<String>;
}


// used for full text search engine
pub trait FullText {
    fn get_content(&self) -> Option<String>;
}


pub struct RangeField {
    pub name: String,
    pub value: String
}