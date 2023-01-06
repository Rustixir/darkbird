use simple_wal::LogError;
use std::{io::Error, time::Duration};

mod index;
mod disk_log;
pub mod document;
mod router;

pub use async_trait::async_trait;

pub mod migration;
pub mod persistent_worker;
pub mod storage;

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

#[derive(Debug)]
pub enum SessionResult {
    Closed,
    Timeout,
    Full,
    NoResponse,
    Err(StatusResult),
}

#[allow(dead_code)]
pub enum WorkerState {
    Continue,
    Disconnected,
    Empty,
}

pub enum StorageType {
    // Store to memory
    RamCopies,

    // Store to memory and persist to disk
    DiskCopies,
}

pub struct Options<'a> {
    path: &'a str,
    storage_name: &'a str,
    total_page_size: usize,
    stype: StorageType,
}

impl<'a> Options<'a> {
    pub fn new(
        path: &'a str,
        storage_name: &'a str,
        total_page_size: usize,
        stype: StorageType,
    ) -> Self {
        Options {
            path,
            storage_name,
            total_page_size,
            stype,
        }
    }
}
