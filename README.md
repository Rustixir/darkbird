
![DarkBird](https://github.com/Rustixir/darkbird/blob/main/darkbird.png)

<div align="center">

  <!-- Downloads -->
  <a href="https://crates.io/crates/darkbird">
    <img src="https://img.shields.io/crates/d/darkbird.svg?style=flat-square"
      alt="Download" />
  </a>
</div>


**DarkBird is a Document oriented, realtime, in-memory storage , 
highly optimized for retrieve document very fast with
indexing and taging feature also persist data 
to disk to avoid loss any data**





The darkbird provides the following:

* **Persistent** - use **Non-Blocking** write-ahead-logging engine for persistency data, 
  also store data to multiple pages with total_page_size
  


* **In-memory** - whole data stored in-memory 
  with two mode ( **DiskCopies** , **RamCopies** )
  both stored in-memory but DiskCopies persistent data to disk and
  after restart, darkbird **load whole data to memory**



* **Concurrency** - darkbird use one of best high-concurrent HashMap (DashMap)[https://github.com/xacrimon/conc-map-bench]
  and **you don't need use Mutex/RwLock for sync between thread,
  storage is complete safe to shared between thread**


* **Indexing**  - darkbird support indexing, can even dynamically
  decision about which fields in document be indexed.
  and each key must be unique else return Duplicate error 


* **FullText Search** - darkbird added InvertedIndex
  from version 3.5.0 for supports FullText Search operation 


* **Materialized View** - provide trait for document model
  `doc.filter(...)` that return `None` or `Some(view_name)`
  when document inserting to storage, 
  if exist (returned) `Some(view_name)` just store document key,
  **Darkbird is first class support materialized view** because
  move cost of heavy operation from search time to insert time 
  target of darkbird is ultra fast retrieving data for RealTime system.

* **Taging** -  each document can have multiple tags
  And one tag can refer to many documents
  The tag is great for indexing groups of documents
  To retrieve very fast by key. For example
  Players of a club, Developers of a company, etc



* **Migration** - darkbird storage model is (Key, Document)
  if you want change Document Model, can use `migration::run` 
  for **change all (Key, Document) already exist in disk**
  this module should be use before storage opened


* **copy/load to external database** - copy storage to (postgres/cassandra/scylla) 
  and load from that 



* **Event Handling** - can subscribe any channel you want to storage, they
  receive events


## Vsn 2.0.0

*  **Improve Performance** 

*  **Persistent** Copy whole data to Database and load from that 



## Vsn 3.0.0

*  **Document model must implement three trait from this vsn**

*  **Indexing** 

*  **Taging** 

*  **Range** 
  range is like indexing but each key can ref to many documents
  also can do range query over indexes to retrieve documents.


## Vsn 3.5.0

* **FullText Search** provide three api 
  `insert_content(document_key, content)` 
  `remove_content(document_key, content)` 
  `search(...)`()

## Vsn 4.0.0

* **Materialized View** 
  from this version Document model must impl MaterializedView trait
  that call `doc.filter()` to get None or ViewName to store on view
  darkbird extract from doc model for remove or insert operation
  and provide one api for get view models `storage.fetch_view(...)`

* **&str instead of &String**
  `&str` is better for calling storage api
  because can call with (static str) param,
  and better performance until use `&String::From("")`

* **All example updates**
  and from this point I try to add features to be compatible with before

Examples
=============

The complete Examples on [Link](https://github.com/Rustixir/darkbird/tree/main/example).


Crate
=============
```
darkbird = "4.0.0"
```



My Plans
=============

1. write great and complete document
   to everyone can know about architecture.


2. **Key expiry** like redis.


3. **Distributing** 
