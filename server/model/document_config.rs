

// these are fields name of document
pub struct DocumentConfig {
    pub tag: String,
    // used for unique index
    pub index: Vec<String>,
    // used for grouping index
    pub tags: Vec<String>,
    // used for range index
    pub range: Vec<String>,
    // used for fulltext search
    pub content: String
}
