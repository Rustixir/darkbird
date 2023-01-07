
![DarkBird](https://github.com/Rustixir/darkbird/blob/main/darkbird.png)

<div align="center">

  <!-- Downloads -->
  <a href="https://crates.io/crates/darkbird">
    <img src="https://img.shields.io/crates/d/darkbird.svg?style=flat-square"
      alt="Download" />
  </a>
</div>


**DarkBird is a Document oriented, concurrency, in-memory Storage, 
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
  get storage event (```RQuery<Key, Document>, Subscribed(tokio::mpsc::Sender(Event<key, document>))```)
 


## Vsn 2.0.0

*  **Improve Performance** 

*  **Persistent** Copy whole data to Database and load from that 



## Vsn 3.0.0

*  **Document model must implement three trait from this vsn**

*  **Indexing** 

*  **Taging** 

*  **Range** 
  range is like indexing but each key can ref to many documents
  also can do range query over indexes to retrieve document.



Examples
=============

The complete Examples on [Link](https://github.com/Rustixir/darkbird/tree/main/example).



Crate
=============
```
darkbird = "2.0.0"
```
