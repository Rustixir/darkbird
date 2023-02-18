use std::{path::Path, fs, marker::PhantomData};
use std::hash::Hash;

use serde::{Serialize, de::DeserializeOwned};
use simple_wal::LogFile;

use crate::RQuery;

use super::disk_log::DEFAULT_PAGE_SIZE;
use super::memory_page::MemoryPage;




pub enum Sync<'a> {
    
    Overwrite, // Create seperate file and write result to there and remove old version.
    
    New(&'a str) //  Create seperate file and write result to there.
}



#[derive(Clone)]
pub enum Format {
    Bson, 
    Bincode
}


enum Recovery {
    Recoverable(Metadata),
    UnRecoverable(String)
}

struct Metadata {
    original_filename:  String,
    currepted_filename: String,
    err: String
}


pub struct PageProcessor<'a, OldKey, OldDoc, NewKey, NewDoc> {
    root: &'a str,
    source_name: &'a str,
    source_total_page_size: usize,
    sync_name: Sync<'a>,
    vacuum: bool,
    handler: fn(RQuery<OldKey, OldDoc>) -> RQuery<NewKey, NewDoc>,

    phan_old_key: PhantomData<OldKey>,
    phan_old_doc: PhantomData<OldDoc>,
    phan_new_key: PhantomData<NewKey>,
    phan_new_doc: PhantomData<NewDoc>
}

impl<'a, OldKey, OldDoc, NewKey, NewDoc> PageProcessor<'a, OldKey, OldDoc, NewKey, NewDoc> 
where
    OldKey: Serialize + DeserializeOwned + Hash + Eq + PartialEq,
    OldDoc: Serialize + DeserializeOwned,
    NewKey: Serialize + DeserializeOwned + Hash + Eq +,
    NewDoc: Serialize + DeserializeOwned
{
    
    pub fn new(root: &'a str, source_name: &'a str, source_total_page_size: usize,
               sync_name: Sync<'a>, vacuum: bool, 
               handler: fn(RQuery<OldKey, OldDoc>) -> RQuery<NewKey, NewDoc>) -> Self 
    {
        PageProcessor {
            root,
            source_name, 
            source_total_page_size,
            sync_name,
            vacuum,
            handler,
            phan_old_key: PhantomData,
            phan_old_doc: PhantomData,
            phan_new_key: PhantomData,
            phan_new_doc: PhantomData,
        }
    }


    pub fn start(self) -> Result<(), String> {
        match self.internal_start() {
            Ok(_) => Ok(()),
            Err(Recovery::UnRecoverable(e)) => return Err(e),
            Err(Recovery::Recoverable(Metadata { original_filename, currepted_filename, mut err })) => {
                if let Sync::Overwrite = self.sync_name {
                    if let Err(e) = fs::remove_file(&currepted_filename) {
                        err.push_str("__");
                        err.push_str(&e.to_string());
                    }
                    if let Err(e) = fs::rename(original_filename, &currepted_filename) {
                        err.push_str("__");
                        err.push_str(&e.to_string());
                    }
                }

                return Err(err)
            }
        }
    }


    fn internal_start(&self) -> Result<(), Recovery> {
        
        let total_page_size =  if self.source_total_page_size < DEFAULT_PAGE_SIZE { DEFAULT_PAGE_SIZE } else { self.source_total_page_size };
        let source_path = format!("{}/{}", self.root, self.source_name);
        let sync_path   = format!("{}/{}", self.root, self.sync_name());

        // not exist source_path
        if !Path::new(&source_path).is_dir() {
            return Err(Recovery::UnRecoverable("folder not exist".to_owned()))
        }


        if !Path::new(&sync_path).is_dir() {
            if let Err(e) = fs::create_dir(&sync_path) {
                return Err(Recovery::UnRecoverable(e.to_string()))
            }
        }

        // page_index
        let mut page_index = 1;

        loop {

            let page_pointer = total_page_size * page_index;
            let source_name = filename_factory(&source_path, page_pointer);

            page_index += 1;


            // filename not exist return Ok
            if !Path::new(&source_name).is_file() {
                return Ok(())
            }


            // prepare source_page_name
            let source_page_name = match self.sync_name {
                Sync::Overwrite => {
                    let rename = filename_factory_rename(&source_path, page_pointer);
                    fs::rename(&source_name, &rename).unwrap();
                    rename
                }
                Sync::New(_) => {
                    source_name.to_owned()
                }
            };


            // open source_page
            let mut source_page = match LogFile::open(&source_page_name) {
                Ok(log) => log,
                Err(e) => {
                    let meta = Metadata {
                        original_filename: source_page_name.to_owned(),
                        currepted_filename: source_name,
                        err: e.to_string(),
                    };
                    return Err(Recovery::Recoverable(meta))
                }
            };



            // prepare sync_page_name
            let sync_page_name = filename_factory(&sync_path, page_pointer);
            

            // open sync_page
            let mut sync_page = match LogFile::open(sync_page_name) {
                Ok(log) => log,
                Err(e) => {
                    let meta = Metadata {
                        original_filename: source_page_name.to_owned(),
                        currepted_filename: source_name.to_owned(),
                        err: e.to_string(),
                    };
                    return Err(Recovery::Recoverable(meta))
                }
            };


            let source_pager_iter = match source_page.iter(..) {
                Ok(iter) => iter,
                Err(e) => {
                    let meta = Metadata {
                        original_filename: source_page_name.to_owned(),
                        currepted_filename: source_name.to_owned(),
                        err: e.to_string(),
                    };
                    return Err(Recovery::Recoverable(meta))
                }
            };


            let mut memory_page = MemoryPage::new();

            for bytes in source_pager_iter {
                match bytes {
                    Err(e) => {
                        let meta = Metadata {
                            original_filename: source_page_name.to_string(),
                            currepted_filename: source_name.to_string(),
                            err: e.to_string(),
                        };
                        return Err(Recovery::Recoverable(meta))
                    }
                    Ok(raw_qline) => {

                        // Deserialize rquery
                        let old_query: RQuery<OldKey, OldDoc> = match bincode::deserialize(&raw_qline) {
                            Ok(res) => res,
                            Err(e) => {
                                let meta = Metadata {
                                    original_filename: source_page_name.to_owned(),
                                    currepted_filename: source_name.to_owned(),
                                    err: e.to_string(),
                                };
                                return Err(Recovery::Recoverable(meta))
                            }
                        };

                        // transform
                        let new_query = (&self.handler)(old_query);


                        if self.vacuum {

                           memory_page.stash(new_query);

                        } else {

                            // serialize
                            let mut bytes = bincode::serialize(&new_query).unwrap();


                            // write to sync
                            if let Err(e) = sync_page.write(&mut bytes) {
                                let meta = Metadata {
                                    original_filename: source_page_name.to_owned(),
                                    currepted_filename: source_name.to_owned(),
                                    err: e.to_string(),
                                };
                                return Err(Recovery::Recoverable(meta))
                            }
                        }
                    }
                }
            }

            if self.vacuum {
                for (_, rquery) in memory_page.get_page().into_iter() {
                    
                    // serialize
                    let mut bytes = bincode::serialize(&rquery).unwrap();

                    // write to sync
                    if let Err(e) = sync_page.write(&mut bytes) {
                        let meta = Metadata {
                            original_filename: source_page_name.to_owned(),
                            currepted_filename: source_name.to_owned(),
                            err: e.to_string(),
                        };
                        return Err(Recovery::Recoverable(meta))
                    }
                }
            }

            // flush if occur error return 
            if let Err(e) = sync_page.flush() {
                let meta = Metadata {
                    original_filename: source_page_name.to_owned(),
                    currepted_filename: source_name.to_owned(),
                    err: e.to_string(),
                };
                return Err(Recovery::Recoverable(meta))
            }

            if let Sync::Overwrite = self.sync_name {
                let _ = fs::remove_file(source_page_name);
            }

        }

    }



    fn sync_name(&self) -> &'a str {
        match self.sync_name {
            Sync::Overwrite => self.source_name,
            Sync::New(name) => name,
        }
    }

}



#[inline]
fn filename_factory_rename(source_path: &str, page_pointer: usize) -> String {
    format!("{}/rpage-{}.LOG", &source_path, page_pointer)
}

#[inline]
fn filename_factory(source_path: &str, page_pointer: usize) -> String {
    format!("{}/page-{}.LOG", &source_path, page_pointer)
}
