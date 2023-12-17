use serde::{Deserialize, Serialize};
use simple_wal::LogError;
use std::{io::Error, time::Duration};

pub mod database;
pub mod document;
mod index;
pub mod persistent_worker;
mod router;
pub mod schema;
pub mod storage;
pub mod storage_redis;
pub mod storage_vector;
mod storage_vector_test;
pub mod vector;
pub mod wal;

pub use async_trait::async_trait;

pub static TIMEOUT: Duration = Duration::from_secs(5);

pub use storage::{Event, RQuery};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Status {
    SenderNotFound,
    SendersRepetive,
}

#[derive(Debug)]
pub enum StatusResult {
    LogErr(LogError),
    IoError(Error),
    End,
    ReporterIsOff,
    Err(String),
    Duplicate,
}

impl ToString for StatusResult {
    fn to_string(&self) -> String {
        match self {
            StatusResult::LogErr(e) => e.to_string(),
            StatusResult::IoError(e) => e.to_string(),
            StatusResult::End => "End".to_string(),
            StatusResult::ReporterIsOff => "ReporterIsOff".to_string(),
            StatusResult::Err(e) => e.to_string(),
            StatusResult::Duplicate => "Duplicate".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum SessionResult {
    Closed,
    Timeout,
    Full,
    NoResponse,
    DataStoreNotFound,
    UnImplement,
    Err(StatusResult),
}

impl ToString for SessionResult {
    fn to_string(&self) -> String {
        match self {
            SessionResult::Closed => "Closed".to_string(),
            SessionResult::Timeout => "Timeout".to_string(),
            SessionResult::Full => "Full".to_string(),
            SessionResult::NoResponse => "NoResponse".to_string(),
            SessionResult::DataStoreNotFound => "DataStoreNotFound".to_string(),
            SessionResult::UnImplement => "UnImplement".to_string(),
            SessionResult::Err(e) => e.to_string(),
        }
    }
}

#[allow(dead_code)]
pub enum WorkerState {
    Continue,
    Disconnected,
    Empty,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum StorageType {
    // Store to memory
    RamCopies,

    // Store to memory and persist to disk
    DiskCopies,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct Options<'a> {
    pub path: &'a str,
    pub storage_name: &'a str,
    pub total_page_size: usize,
    pub stype: StorageType,
    pub off_reporter: bool,
}

impl<'a> Options<'a> {
    pub fn new(
        path: &'a str,
        storage_name: &'a str,
        total_page_size: usize,
        stype: StorageType,
        off_reporter: bool,
    ) -> Self {
        Options {
            path,
            storage_name,
            total_page_size,
            stype,
            off_reporter,
        }
    }
}

impl<'a> Into<Config> for Options<'a> {
    fn into(self) -> Config {
        Config {
            path: self.path.to_owned(),
            storage_name: self.storage_name.to_owned(),
            total_page_size: self.total_page_size.to_owned(),
            stype: self.stype.to_owned(),
            off_reporter: self.off_reporter.to_owned(),
        }
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub path: String,
    pub storage_name: String,
    pub total_page_size: usize,
    pub stype: StorageType,
    pub off_reporter: bool,
}

impl Config {
    pub fn new(
        path: String,
        storage_name: String,
        total_page_size: usize,
        stype: StorageType,
        off_reporter: bool,
    ) -> Self {
        Config {
            path,
            storage_name,
            total_page_size,
            stype,
            off_reporter,
        }
    }


}
