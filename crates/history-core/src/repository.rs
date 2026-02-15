use crate::model::*;
use async_trait::async_trait;

#[async_trait]
pub trait HistoryRepository: Send + Sync {
    // Command history
    async fn list_command_history(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CommandHistory>, String>;
    async fn add_command(
        &self,
        command: &str,
        session_id: Option<&str>,
        connection_id: Option<i64>,
    ) -> Result<i64, String>;
    async fn clear_command_history(&self) -> Result<(), String>;
    async fn delete_command_history_entry(&self, id: i64) -> Result<bool, String>;

    // Path history
    async fn list_path_history(
        &self,
        connection_id: Option<i64>,
        limit: i64,
    ) -> Result<Vec<PathHistory>, String>;
    async fn add_path(&self, path: &str, connection_id: Option<i64>) -> Result<i64, String>;
    async fn clear_path_history(&self) -> Result<(), String>;

    // Favorite paths
    async fn list_favorite_paths(
        &self,
        connection_id: Option<i64>,
    ) -> Result<Vec<FavoritePath>, String>;
    async fn create_favorite_path(
        &self,
        name: &str,
        path: &str,
        connection_id: Option<i64>,
    ) -> Result<i64, String>;
    async fn update_favorite_path(
        &self,
        id: i64,
        name: &str,
        path: &str,
        connection_id: Option<i64>,
    ) -> Result<bool, String>;
    async fn delete_favorite_path(&self, id: i64) -> Result<bool, String>;
    async fn mark_favorite_path_used(&self, id: i64) -> Result<bool, String>;
}
