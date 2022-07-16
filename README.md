
![DarkBird](https://github.com/Rustixir/darkbird/blob/main/darkbird.png)

<div align="center">

  <!-- Downloads -->
  <a href="https://crates.io/crates/darkbird">
    <img src="https://img.shields.io/crates/d/darkbird.svg?style=flat-square"
      alt="Download" />
  </a>
</div>


**DarkBird is a Document oriented, high concurrency in-memory Storage, 
also persist data to disk to avoid loss any data**





The darkbird provides the following:

* **Persistent** - use **Non-Blocking** wal engine for persist data, 
  also store data to multiple pages by total_page_size
  


* **In-memory** - whole data stored in-memory 
  with two mode ( **DiskCopies** , **RamCopies** )
  both stored in-memory but DiskCopies persist data to disk and
  after restart, darkbird **load whole data to memory**




* **Concurrency** - darkbird use one of best high-concurrent HashMap (DashMap)[https://github.com/xacrimon/conc-map-bench]
  and **you don't need use Mutex/RwLock for sync between thread,
  storage is complete safe to shared between thread**





* **Migration** - Darkbird storage model is (Key, Document)
  if you want change Document Model, can use `migration::run` 
  for **change all (Key, Document) already exist in disk**
  this module should be use before storage opened


* **Persist to database** - copy storage to (postgres/cassandra/scylla) 
  and load from that 



* **Event Handling** - can subscribe any channel you want to storage, they
  get storage event (```RQuery<Key, Document>, Subscribed(tokio::mpsc::Sender(Event<key, document>))```)
 


## Vsn 2.0.0

*  **Improve Performance** 

*  **Persistent** Copy whole data to Database and load from that 

Examples
=============

The complete Examples on [Link](https://github.com/Rustixir/darkbird/tree/main/example).



Crate
=============
```
darkbird = "2.0.0"
```
