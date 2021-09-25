use async_session::MemoryStore;

pub fn init_session_store() -> MemoryStore {
    MemoryStore::new()
}
