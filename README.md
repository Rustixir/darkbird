![DarkBird](https://github.com/Rustixir/darkbird/blob/main/darkbird.png)

<div align="center">
  <!-- Downloads -->
  <a href="https://crates.io/crates/darkbird">
    <img src="https://img.shields.io/crates/d/darkbird.svg?style=flat-square"
      alt="Download" />
  </a>
</div>

DarkBird is a _document-oriented_, _real-time_, _in-memory_ storage solution optimized for fast **document retrieval**.

## Features
- **Persistent**: Uses a _non-blocking_ write-ahead-logging engine for data persistence, storing data to multiple pages.
- **In-memory**: Data is stored in memory, with two modes (_DiskCopies_, _RamCopies_), the first persisting data to disk and reloading the data into memory after restart.
- **Concurrency**: Uses a high-concurrent HashMap ([_DashMap_](https://github.com/xacrimon/conc-map-bench)) and doesn't require Mutex/RwLock for thread synchronization.
- **Indexing**: Supports indexing, allowing for dynamic decision-making about which document fields to index.
- **Full-text search**: Supports full-text search operations since version 3.5.0.
- **Materialized view**: Provides a trait for the document model `(doc.filter(...))` that returns `Some(view_name)` or `None` when a document is inserted into the storage.
- **Tagging**: Each document can have multiple tags, and one tag can refer to many documents, making it great for indexing groups of documents for fast retrieval by key.
- **Migration**: The storage model is (Key, Document), and you can use `migration::run` to change the existing (Key, Document) data on disk before opening the storage.
- **External database support**: Copy storage data to Postgres/Cassandra/Scylla and load from it.
- **Event handling**: Subscribe to any channel to receive events.

## Crate

```
darkbird = "4.0.0"
```

## Examples
See the complete examples [here](https://github.com/Rustixir/darkbird/tree/main/example).

## Versions
- **2.0.0**: Improved _performance_ and added _persistent copy_ of whole data to a database.
- **3.0.0**: Implemented _indexing_, _tagging_, and _range queries_. **Document model must implement tree trait from this version**
- **3.5.0**: Added _full-text search API_ (`insert_content(document_key, content)`, `remove_content(document_key, content)`, and `search(...)`).
- **4.0.0**: Added _materialized view_ support. Document models must implement the MaterializedView trait, and API is provided to fetch view models. Uses `&str` instead of `&String` for better performance and API compatibility. All examples are updated.

## Future plans
- Write comprehensive **documentation** to explain the architecture.
- Add **key expiry** similar to Redis.
- **Distribution**.
