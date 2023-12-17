![DarkBird](https://github.com/Rustixir/darkbird/blob/main/darkbird.png)

<div align="center">
  <!-- Downloads -->
  <a href="https://crates.io/crates/darkbird">
    <img src="https://img.shields.io/crates/d/darkbird.svg?style=flat-square"
      alt="Download" />
  </a>
</div>

DarkBird is a _document-oriented_, _real-time_, _in-memory_ database solution optimized for fast **document retrieval**.

## Features
- **Database level**: darkbird was storage, but from ( vsn 5.0.3 ) is full-featured database
because provide Schema for building database and all operation do with database layer
- **Persistent**: Uses a _non-blocking_ write-ahead-logging engine for data persistence, storing data to multiple pages.
- **In-memory**: Data is stored in memory, with two modes (_DiskCopies_, _RamCopies_), the first persisting data to disk and reloading the data into memory after restart.
- **Concurrency**: Uses a high-concurrent HashMap ([_DashMap_](https://github.com/xacrimon/conc-map-bench)) and doesn't require Mutex/RwLock for thread synchronization.
- **Vector**: darkbird provide a vector engine for storing and searching vectors
- **Indexing**: Supports indexing, allowing for dynamic decision-making about which document fields to index.
- **Full-text search**: Supports full-text search operations since version 3.5.0.
- **Materialized view**: Supports materialized view
- **Tagging**: Each document can have multiple tags, and one tag can refer to many documents, making it great for indexing groups of documents for fast retrieval by key.
- **Expiration**: from vsn 6.0.0 support key expiry.
- **Atomic Operation**: from vsn 6.0.0 support Atomic Operation (just like redis setNx)
- **Migration**: The storage model is (Key, Document), and you can use `migration` to change the existing (Key, Document) data on disk before opening the storage.
- **Backup / Restore** from vsn-6.1.0 support Backup/Restore
- **External database support**: Copy storage data to Postgres/Cassandra/Scylla and load from it.
- **Event handling**: Subscribe to any channel to receive events.

## Crate

```
darkbird = "6.2.1"
```

## Examples
- See the complete examples [here](https://github.com/Rustixir/darkbird/tree/main/example).
- This repo is Movies store service with (darkbird + actix-web) [here](https://github.com/Rustixir/darkapp/).


## Versions
- **2.0.0**: Improved _performance_ and added _persistent copy_ of whole data to a database.
- **3.0.0**: Implemented _indexing_, _tagging_, and _range queries_. **Document model must implement tree trait from this version**
- **3.5.0**: Added full-text search API
- **4.0.0**: Added _materialized view_ support. Document models must implement the MaterializedView trait, and API is provided to fetch view models. Uses `&str` instead of `&String` for better performance and API compatibility. All examples are updated.
- **5.0.1**: migrated from Storage to Database world with Schema builder
and Database layer to do all operation also is compatible with older version 
- **5.0.2**: fixedbug persist indexing and changed fullText search api for a bug 
all examples updates
- **5.0.3**: fixedbug loader
- **6.0.0**: added another storage Engine for supporting:
  atomic operation (just like redis setNx), expiration and simpler api  
- **6.0.1**: Backup/Restore _ new migration component (recover self if occure error)
- **6.2.1**: Vector Engine

<a href="https://www.buymeacoffee.com/xnALmpJF9a" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/default-yellow.png" alt="Buy Me A Coffee" height="41" width="174"></a>

