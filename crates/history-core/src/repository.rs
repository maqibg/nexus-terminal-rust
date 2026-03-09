use crate::model::*;
use async_trait::async_trait;
use shared_utils::StorageError;

#[async_trait]
pub trait HistoryRepository: Send + Sync {
    // Command history
    async fn list_command_history(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CommandHistory>, StorageError>;
    async fn add_command(
        &self,
        command: &str,
        session_id: Option<&str>,
        connection_id: Option<i64>,
    ) -> Result<i64, StorageError>;
    async fn clear_command_history(&self) -> Result<(), StorageError>;
    async fn delete_command_history_entry(&self, id: i64) -> Result<bool, StorageError>;

    // Path history
    async fn list_path_history(
        &self,
        connection_id: Option<i64>,
        limit: i64,
    ) -> Result<Vec<PathHistory>, StorageError>;
    async fn add_path(&self, path: &str, connection_id: Option<i64>) -> Result<i64, StorageError>;
    async fn clear_path_history(&self) -> Result<(), StorageError>;

    // Favorite paths
    async fn list_favorite_paths(
        &self,
        connection_id: Option<i64>,
    ) -> Result<Vec<FavoritePath>, StorageError>;
    async fn create_favorite_path(
        &self,
        name: &str,
        path: &str,
        connection_id: Option<i64>,
    ) -> Result<i64, StorageError>;
    async fn update_favorite_path(
        &self,
        id: i64,
        name: &str,
        path: &str,
        connection_id: Option<i64>,
    ) -> Result<bool, StorageError>;
    async fn delete_favorite_path(&self, id: i64) -> Result<bool, StorageError>;
    async fn mark_favorite_path_used(&self, id: i64) -> Result<bool, StorageError>;
}
