// Canonical implementations of user management interfaces
// Single responsibility, dependency injection, medical device compliant

pub mod file_user_storage;
pub mod memory_user_storage;
pub mod global_user_storage;
pub mod file_session_storage;

pub use file_user_storage::FileUserStorage;
pub use memory_user_storage::MemoryUserStorage;
pub use global_user_storage::GlobalUserStorage;
pub use file_session_storage::FileSessionStorage;
