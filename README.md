
![BlackBird](https://github.com/Rustixir/BlackBird/blob/master/New%20Project.png)



Blackbird is an high concurrency in-memory Storage also 
persist data to disk to avoid loss any data


The blackbird provides the following:

* **Persistent** - use **Non-Blocking** wal engine for persist data, 
  also store data to multiple pages by total_page_size
  


* **In-memory** - whole data stored in-memory and have two mode ( DiskCopies, RamCopies )
  both stored in-memory but DiskCopies persist data to disk and
  after restart, blackbird load whole data to memory 



* **Concurrency** - blackbird use one of best high-concurrent HashMap (DashMap)[https://github.com/xacrimon/conc-map-bench]
  and you dont need use Mutex/RwLock for sync between thread,
  storage is complete safe to shared between thread




* **Migration** - Blackbird storage model is (Key, Document)
  if you want change Document Model, can use `migration::run` to change all (Key, Document)
  already exist in disk, this module should use before storage opened




* **Event Handling** - can subscribe any channel you want to storage, they
  get storage event ('Insert(key, document)', 'Remove(key)', 'Subscribed(tokio::mpsc::Sender(Event<key, document>))')
 


Examples
=============

The complete Examples on [Link](https://github.com/Rustixir/blackbird/tree/main/example).



Crate
=============
Link []()
