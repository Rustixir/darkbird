use tokio::sync::Notify;
use tokio::time::{self, Duration, Instant};

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::hash::Hash;


#[derive(Debug)]
pub struct DbDropGuard<K, Doc> 
where
    Doc: Clone + Send + Sync + 'static,
    K:  Clone
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Send
        + 'static
{
    storage: RedisStorage<K, Doc>,
}

#[derive(Debug, Clone)]
pub struct RedisStorage<K, Doc> 
where
    Doc: Clone + Send + Sync + 'static,
    K:  Clone
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Send
        + 'static
{
    shared: Arc<Shared<K, Doc>>,
}

#[derive(Debug)]
struct Shared<K, Doc> 
where
    Doc: Clone + Send + Sync + 'static,
    K:  Clone
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Send
        + 'static
{
    state: Mutex<State<K, Doc>>,
    background_task: Notify,
}

#[derive(Debug)]
struct State<K, Doc> 
where
    Doc: Clone + Send + Sync + 'static,
    K:  Clone
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Send
        + 'static
{
    
    entries: HashMap<K, Entry<Doc>>,
    expirations: BTreeMap<(Instant, u64), K>,
    next_id: u64,
    shutdown: bool,
}


#[derive(Debug)]
struct Entry<Doc> {
    id: u64,
    data: Arc<Doc>,
    expires_at: Option<Instant>,
}

impl<K, Doc> DbDropGuard<K, Doc> 
where
    Doc: Clone + Send + Sync + 'static,
    K:  Clone
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Send
        + 'static
{
    pub(crate) fn new() -> DbDropGuard<K, Doc> {
        DbDropGuard { storage: RedisStorage::new() }
    }


    pub(crate) fn storage(&self) -> RedisStorage<K, Doc> {
        self.storage.clone()
    }
}

impl<K, Doc> Drop for DbDropGuard<K, Doc> 
where
    Doc: Clone + Send + Sync + 'static,
    K:  Clone
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Send
        + 'static
{
    fn drop(&mut self) {
        self.storage.shutdown_purge_task();
    }
}

impl<K, Doc> RedisStorage<K, Doc> 
where
    Doc: Clone + Send + Sync + 'static,
    K:  Clone
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Send
        + 'static
{

    pub fn new() -> RedisStorage<K, Doc> {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                entries: HashMap::new(),
                expirations: BTreeMap::new(),
                next_id: 0,
                shutdown: false,
            }),
            background_task: Notify::new(),
        });

        tokio::spawn(purge_expired_tasks(shared.clone()));

        RedisStorage { shared }
    }

    pub fn get(&self, key: &K) -> Option<Arc<Doc>> {
    
        let state = self.shared.state.lock().unwrap();
        state.entries.get(key).map(|entry| entry.data.clone())
    }

    pub fn set(&self, key: K, value: Doc, expire: Option<Duration>) {
        let mut state: std::sync::MutexGuard<'_, State<K, Doc>> = self.shared.state.lock().unwrap();

        
        let id = state.next_id;
        state.next_id += 1;

    
        let mut notify = false;
        let expires_at = expire.map(|duration| {
            
            let when = Instant::now() + duration;
            notify = state
                .next_expiration()
                .map(|expiration| expiration > when)
                .unwrap_or(true);

        
            state.expirations.insert((when, id), key.clone());
            when
        });

        let prev = state.entries.insert(
            key,
            Entry {
                id,
                data: Arc::new(value),
                expires_at,
            },
        );

        if let Some(prev) = prev {
            if let Some(when) = prev.expires_at {
                state.expirations.remove(&(when, prev.id));
            }
        }

        drop(state);

        if notify {
            
            self.shared.background_task.notify_one();
        }
    }

    pub fn set_nx(&self, key: K, value: Doc, expire: Option<Duration>) -> bool {
        let mut state = self.shared.state.lock().unwrap();

        if state.entries.contains_key(&key) {
            return false
        }

        let id = state.next_id;
        state.next_id += 1;


        let mut notify = false;

        let expires_at = expire.map(|duration| {
            let when = Instant::now() + duration;
            notify = state
                .next_expiration()
                .map(|expiration| expiration > when)
                .unwrap_or(true);

            state.expirations.insert((when, id), key.clone());
            when
        });

        let prev = state.entries.insert(
            key,
            Entry {
                id,
                data: Arc::new(value),
                expires_at,
            },
        );

        if let Some(prev) = prev {
            if let Some(when) = prev.expires_at {
                
                state.expirations.remove(&(when, prev.id));
            }
        }

        drop(state);

        if notify {
            self.shared.background_task.notify_one();
        }

        return true;
    }

    pub fn del(&self, key: &K) {
        let mut state = self.shared.state.lock().unwrap();
        state.entries.remove(key);
    }

 
    pub fn len(&self) -> usize {
        let mut state = self.shared.state.lock().unwrap();
        return state.entries.len()
    }

    fn shutdown_purge_task(&self) {

        let mut state = self.shared.state.lock().unwrap();
        state.shutdown = true;

        drop(state);
        self.shared.background_task.notify_one();
    }
}

impl<K, Doc> Shared<K, Doc> 
where
    Doc: Clone + Send + Sync + 'static,
    K:  Clone
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Send
        + 'static
{
    
    fn purge_expired_keys(&self) -> Option<Instant> {
        let mut state = self.state.lock().unwrap();

        if state.shutdown {
            return None;
        }

        let state = &mut *state;
        let now = Instant::now();

        while let Some((&(when, id), key)) = state.expirations.iter().next() {
            if when > now {
                return Some(when);
            }

            state.entries.remove(key);
            state.expirations.remove(&(when, id));
        }

        None
    }

    fn is_shutdown(&self) -> bool {
        self.state.lock().unwrap().shutdown
    }
}

impl<K, Doc> State<K, Doc> 
where
    Doc: Clone + Send + Sync + 'static,
    K:  Clone
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Send
        + 'static
{
    fn next_expiration(&self) -> Option<Instant> {
        self.expirations
            .keys()
            .next()
            .map(|expiration| expiration.0)
    }
}


async fn purge_expired_tasks<K, Doc>(shared: Arc<Shared<K, Doc>>) 
where
    Doc: Clone + Send + Sync + 'static,
    K:  Clone
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Send
        + 'static
{
    while !shared.is_shutdown() {
        if let Some(when) = shared.purge_expired_keys() {
            
            tokio::select! {
                _ = time::sleep_until(when) => {}
                _ = shared.background_task.notified() => {}
            }
        } else {
            
            shared.background_task.notified().await;
        }
    }
}