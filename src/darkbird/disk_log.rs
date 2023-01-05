
pub const DEFAULT_PAGE_SIZE: usize   = 5000; 
pub const DISKLOG_BUFFER_SIZE: usize = 1000;


struct TmpLogStruct {
    path: String,
    log: LogFile,
    current_page_index: usize
}    



enum Request {

    Record(Vec<u8>),

    GetPage {
        page_index: usize, 
        dst: oneshot::Sender<Result<LogFile, StatusResult>>,
    },
}


pub struct DiskLog {
    context: Context
}

impl DiskLog {
    pub fn open (path: &str, 
                 table_name: &str, 
                 total_page_size: usize) -> Result<Self, LogError>  
    {
        match Context::open(path, table_name, total_page_size) {
            Ok(context) => {
                Ok(DiskLog {
                    context
                })        
            }
            Err(e) => {
                return Err(e)
            }
        }
        
    }

    pub fn run_service(mut self) -> Session {
        let (sx, mut rx) = mpsc::channel(DISKLOG_BUFFER_SIZE);
        std::thread::spawn(move || {

            let mut worker_state;

            loop {

                // Block until get request
                match self.handle_recv(rx.blocking_recv()) {
                    Ok(w) => {
                        worker_state = w;
                    }
                    Err(e) => {
                        
                        // TODO: expose a metric for IO Errors, or print to stderr
                        eprintln!("{:?}", e);

                        worker_state = WorkerState::Continue;                        

                    }
                }
                
                // loop channel until empty 
                loop {

                    if let WorkerState::Continue = worker_state {
                    
                        // try_recv
                        let try_recv_result = rx.try_recv();

                        // handle_try_recv
                        match self.handle_try_recv(try_recv_result) {
                            Ok(w) => {
                                worker_state = w;
                            }
                            Err(e) => {

                                // TODO: expose a metric for IO Errors, or print to stderr
                                eprintln!("{:?}", e);

                                worker_state = WorkerState::Continue;                            
                            }
                        }

                    } else {

                        // worker_state is (Disconnect or Empty)
                        break;
                    }
                    
                }

                // Flush
                let _= self.context.log.flush();


                // if worker_state was disconnect terminate
                if let WorkerState::Disconnected = worker_state {
                    return
                }
            }
        });

        Session::new(sx)
    }
    
    fn handle_recv(&mut self, op: Option<Request>) -> Result<WorkerState, StatusResult> {
        match op {
            Some(req) => {
                match req {
                    Request::Record(mut bytes) => {
                        // Log
                        match self.context.write_to_disk(&mut bytes) {
                            Ok(_) => Ok(WorkerState::Continue),
                            Err(e) => Err(e),
                        }
                    }
                    Request::GetPage { page_index, dst } => {
                        
                        let filename = self.context.find_filename(page_index);
                        
                        // Check file exist  
                        if Path::new(&filename).is_file() {
                            // open logFile
                            match LogFile::open(filename) {
                                Ok(log) => {
                                    // send logfile
                                    let _ = dst.send(Ok(log));
                                    Ok(WorkerState::Continue)
                                }
                                Err(e) => {

                                    // LogErr is => ("Bad checksum" || "Out of bounds" || "the log is locked"
                                    let _ = dst.send(Err(StatusResult::LogErr(e)));
                                    
                                    return  Ok(WorkerState::Continue)
                                }
                            }

                        // file not found
                        } else {
                            let _ = dst.send(Err(StatusResult::End));
                            Ok(WorkerState::Continue)
                        }
                    }
                }
            }
            None => Ok(WorkerState::Disconnected)
        }
    }

    fn handle_try_recv(&mut self, res: Result<Request, TryRecvError>) -> Result<WorkerState, StatusResult> {
        match res {
            Ok(req) => {
                match req {
                    Request::Record(mut bytes) => {
                        // Log
                        match self.context.write_to_disk(&mut bytes) {
                            Ok(_) => Ok(WorkerState::Continue),
                            Err(e) => Err(e),
                        }
                    }
                    Request::GetPage { page_index, dst } => {
                        
                        let filename = self.context.find_filename(page_index);
                        
                        // Check file exist  
                        if Path::new(&filename).is_file() {
                            // open logFile
                            match LogFile::open(filename) {
                                Ok(log) => {
                                    // send logfile
                                    let _ = dst.send(Ok(log));
                                    Ok(WorkerState::Continue)
                                }
                                Err(e) => {

                                    // LogErr is => ("Bad checksum" || "Out of bounds" || "the log is locked"
                                    let _ = dst.send(Err(StatusResult::LogErr(e)));
                                    
                                    return  Ok(WorkerState::Continue)
                                }
                            }
                            
                        // file not found
                        } else {
                            let _ = dst.send(Err(StatusResult::End));
                            Ok(WorkerState::Continue)
                        }
                    }
                }
            }
            Err(e) => {
                match e {
                    TryRecvError::Empty => Ok(WorkerState::Empty),
                    TryRecvError::Disconnected => Ok(WorkerState::Disconnected)
                }
            }
        }
    }
    
    

}





struct Context {
    log: LogFile,
    path: String,

    // total_page_size is total len of query list per file.LOG  
    total_page_size: usize,

    // used_page is len of reocrd in current_file.LOG 
    used_page: usize,

    // current_page is pointer to current_page
    current_page_index: usize

}
impl Context {

    pub fn open(path: &str, 
                table_name: &str, 
                mut total_page_size: usize) -> Result<Self, LogError> 
    {

        // at-least DEFAULT_PAGE_SIZE Record
        total_page_size =  if total_page_size < DEFAULT_PAGE_SIZE { DEFAULT_PAGE_SIZE } else { total_page_size };
        

        let mut slog = open_last_page(path, table_name, total_page_size);
        
        let used_page = used_page(&mut slog.log);

        Ok(Context{
            log: slog.log,

            path: slog.path,
            
            total_page_size,

            used_page,

            // current_page is pointer to current_page and when move to new page change
            current_page_index: 1
        })
    }
 

    #[inline]
    fn write_to_disk(&mut self, bytes: &mut Vec<u8>) -> Result<(), StatusResult> {
        let sum = self.used_page + 1;

        // if page have free space 
        if sum < self.total_page_size {

            match self.log.write(bytes) {

                Ok(_) => {                    
                    self.used_page = sum;
                    Ok(())
                }
                Err(err) => {
                    return Err(StatusResult::IoError(err))
                }
            }
            
        }
        // if page with this become full
        else if sum == self.total_page_size {
            // write buffer to page
            let res = self.log.write(bytes);

            // if fail return
            if let Err(e) = res {
                return Err(StatusResult::IoError(e))
            }

            // flush to disk because move to next page
            match self.log.flush() {
                Ok(_) => {

                    // ----- move to new page -----
                    self.current_page_index += 1;

                    // create filename for new page
                    let curr_filename = filename_factory(&self.path, self.current_page_index * self.total_page_size);

                    // create new page
                    let log = match LogFile::open(&curr_filename) {
                        Ok(lf) => lf,
                        Err(err) => {
                            return Err(StatusResult::LogErr(err))
                        }
                    };

                    self.used_page = 0;
                    self.log = log;

                    Ok(())
                }
                Err(err) => {
                    return Err(StatusResult::IoError(err))
                }
            }
        }
        // if page is full
        else {

            // flush to disk because move to next page
            if let Err(e) = self.log.flush() {
                // return error
                return Err(StatusResult::IoError(e));
            }

            // ----- move to new page -----
            self.current_page_index += 1;

            // create filename for new page
            let curr_filename = filename_factory(&self.path, self.current_page_index * self.total_page_size);
            
            
            // create new page
            let log = match LogFile::open(&curr_filename) {
                Ok(lf) => lf,
                Err(err) => {
                    return Err(StatusResult::LogErr(err))
                }
            };
        
            self.log = log;       
        
            // write buffer to page
            let res = self.log.write(bytes);
            
            match res {
                Ok(_) => {
                    self.used_page = 1;
                    Ok(())
                }
                Err(err) => {
                    return Err(StatusResult::IoError(err))
                }
            } 

        }
    }


    #[inline]
    fn find_filename(&self, page_index: usize) -> String {
        let s = filename_factory(&self.path, self.total_page_size * page_index);
        
        s
    }

}
 

// -------------------------------------------------


use crate::darkbird::{SessionResult, StatusResult};

use std::time::Duration;
use std::{path::Path, fs};

use simple_wal::{LogFile, LogError};
use tokio::sync::mpsc::error::{TryRecvError, SendTimeoutError};
use tokio::sync::{oneshot, mpsc};


pub enum WorkerState {
    Continue,
    Disconnected,
    Empty
}

pub static TIMEOUT: Duration = Duration::from_secs(5);



#[inline]
fn filename_factory(path: &str, page_pointer: usize) -> String {
    format!("{}/page-{}.LOG", &path, page_pointer)
}

// if not exist directory then is first time run, create dir and a page-1 and open it
// else open latest page exist
fn used_page(log: &mut LogFile) -> usize {

    let mut counter = 0;
    let iter = log.iter(..).unwrap();
    iter.for_each(|_| counter += 1);

    return counter
}

fn open_last_page(path: &str, table_name: &str, total_page_size: usize) -> TmpLogStruct {
     let path = format!("{}/{}", path, table_name);
     // if not exist, (First times is started_service)
     if !Path::new(&path).is_dir() {
        let curr_filename = filename_factory(&path, total_page_size);
        fs::create_dir(&path).unwrap();
        return TmpLogStruct {
            path: path,
            log: LogFile::open(&curr_filename).unwrap(),
            current_page_index: 1
        }
     }
    else {
         // -----------------------------------------
         let mut page_index = 2;
         let mut latest_page = filename_factory(&path, total_page_size); 
         loop {        

            let page_pointer = total_page_size * page_index;
            let filename = filename_factory(&path, page_pointer);
        
            if Path::new(&filename).is_file() {
                latest_page = filename;
                page_index += 1; 
            } 
            else {
                // if not exist page_2
                if page_index == 2 {
                    return TmpLogStruct {
                        path,
                        log: LogFile::open(latest_page).unwrap(),
                        current_page_index: 1
                    }
                } 
                else {
                    return TmpLogStruct {
                        path,
                        log: LogFile::open(latest_page).unwrap(),
                        current_page_index: (page_index - 1)
                    }
                }
            }                
        }
     }

}




// --------------------- Client Code --------------------------


pub struct Session {
    sender: mpsc::Sender<Request>
}

impl Session {
    fn new(sender: mpsc::Sender<Request>) -> Self {
        Session { 
            sender 
        }
    }



    /// checkin a resource
    pub async fn log(&self, record: Vec<u8>) -> Result<(), SessionResult> {

        let res = self.sender.send_timeout(Request::Record(record), TIMEOUT).await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    SendTimeoutError::Timeout(_) => Err(SessionResult::Timeout),
                    SendTimeoutError::Closed(_) => Err(SessionResult::Closed),
                }
            }
        }
    }   

    /// checkout a resource
    pub async fn get_page(&self, index: usize) -> Result<LogFile, SessionResult> {
        
        // create oneshot channel
        let (ask, resp) = oneshot::channel();
        
        // create a request
        let req = Request::GetPage { page_index: index, dst: ask };
        
        
        // send request with timeout (5 seconds)
        let res = self.sender.send_timeout(req, TIMEOUT).await;
           
        match res {
            // Closed
            Err(SendTimeoutError::Closed(_req)) => {
                return Err(SessionResult::Closed)
            }
            // Timeout
            Err(SendTimeoutError::Timeout(_req)) => {
                return Err(SessionResult::Timeout)
            }

            // Sending request was successful
            Ok(_) => {
                // Await for resource
                match resp.await {
                    Ok(res) => {

                        match res {
                            Ok(r) => Ok(r),
                            Err(e) => Err(SessionResult::Err(e)),
                        }
                        
                    }
                    Err(_) => {
                        return Err(SessionResult::NoResponse)
                    }
                }
            }
        }
    }



}