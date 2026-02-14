//! SQLite storage layer: DB pool, migrations, repository implementations.

pub mod migrations;
pub mod pool;
pub mod repo_auth;
pub mod repo_auxiliary;
pub mod repo_connection;
pub mod repo_settings;

pub use pool::{init_pool, SqliteStorage};
pub use repo_auth::SqliteAuthRepo;
pub use repo_auxiliary::{SqliteAuditRepo, SqliteHistoryRepo, SqliteQuickCommandRepo};
pub use repo_connection::SqliteConnectionRepo;
pub use repo_settings::SqliteSettingsRepo;
