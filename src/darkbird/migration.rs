use std::{path::Path, fs};
use super::{disk_log::DEFAULT_PAGE_SIZE, StatusResult};
use serde::{de::DeserializeOwned, Serialize};
use simple_wal::LogFile;

use super::storage::RQuery;



pub fn run<OldKey, OldDocument,
           NewKey, NewDocument, F> 

           (path: &str, 
            storage_name: &str, 
            mut total_page_size: usize,
            handler: F) -> Result<(), StatusResult>
           
where
    OldKey: DeserializeOwned,
    OldDocument: DeserializeOwned,

    NewKey: Serialize + Clone,
    NewDocument: Serialize + Clone,

    F: Fn(RQuery<OldKey, OldDocument>) -> RQuery<NewKey, NewDocument>
{

    // at-least DEFAULT_PAGE_SIZE Record
    total_page_size =  if total_page_size < DEFAULT_PAGE_SIZE { DEFAULT_PAGE_SIZE } else { total_page_size };

    
    let path = format!("{}/{}", path, storage_name);

    // not exist path
    if !Path::new(&path).is_dir() {
        eprintln!("==> Path Not Exist");
        return Err(StatusResult::Err("Path not exist".to_owned()))
    }
    
    // page_index
    let mut page_index = 1;

    loop {

        let page_pointer = total_page_size * page_index;

        let filename_rename = filename_factory_rename(&path, page_pointer);
        let filename = filename_factory(&path, page_pointer);
        
        page_index += 1;
        println!("==> {}", &filename);

        // filename not exist return Ok
        if !Path::new(&filename).is_dir() {
            return Ok(())
        }

        // do rename filename 
        fs::rename(&filename, &filename_rename).unwrap();



        // open filename_rename
        let mut logfile_renamed = match LogFile::open(filename_rename.clone()) {
            Ok(log) => log,
            Err(e) => return Err(StatusResult::LogErr(e))
        };

        // open filename
        let mut logfile = match LogFile::open(filename) {
            Ok(log) => log,
            Err(e) => return Err(StatusResult::LogErr(e))
        };


        let iter_renamed = match logfile_renamed.iter(..) {
            Ok(iter) => iter,
            Err(e) => return Err(StatusResult::LogErr(e)),
        };

        for res in iter_renamed {
            match res {
                Err(e) => return Err(StatusResult::LogErr(e)),
                Ok(qline) => {
                    
                    // Deserialize rquery
                    let old_query: RQuery<OldKey, OldDocument> = bincode::deserialize(&qline).unwrap();
                    
                    // map_to 'old' to 'new' version
                    let new_query = handler(old_query);

                    // write to disk, if return err return back
                    if let Err(e) = logfile.write(&mut bincode::serialize(&new_query).unwrap()) {
                        return  Err(StatusResult::IoError(e))
                    }
                }
            }
        }

        // flush if occur error return 
        if let Err(e) = logfile.flush() {
            return Err(StatusResult::IoError(e))
        }

        let _ = fs::remove_file(filename_rename);

    }

}



#[inline]
fn filename_factory_rename(path: &str, page_pointer: usize) -> String {
    format!("{}/rpage-{}.LOG", &path, page_pointer)
}

#[inline]
fn filename_factory(path: &str, page_pointer: usize) -> String {
    format!("{}/page-{}.LOG", &path, page_pointer)
}
