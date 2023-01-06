
pub trait Document: Indexer + Tags + Range {}

pub trait Indexer {
    fn extract(&self) -> Vec<String>;
}

pub trait Tags {
    fn get_tags(&self) -> Vec<String>;
}

pub trait Range {
    fn get_fields(&self) -> Vec<RangeField>;
}


pub struct RangeField {
    pub name: String,
    pub value: String
}