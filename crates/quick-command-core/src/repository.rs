use crate::model::*;
use async_trait::async_trait;
use shared_utils::StorageError;

#[async_trait]
pub trait QuickCommandRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<QuickCommand>, StorageError>;
    async fn get(&self, id: i64) -> Result<Option<QuickCommand>, StorageError>;
    async fn create(&self, input: &QuickCommandInput) -> Result<i64, StorageError>;
    async fn update(&self, id: i64, input: &QuickCommandInput) -> Result<bool, StorageError>;
    async fn delete(&self, id: i64) -> Result<bool, StorageError>;
    async fn increment_usage(&self, id: i64) -> Result<(), StorageError>;

    // Quick command tags
    async fn list_tags(&self) -> Result<Vec<QuickCommandTag>, StorageError>;
    async fn create_tag(&self, name: &str) -> Result<i64, StorageError>;
    async fn delete_tag(&self, id: i64) -> Result<bool, StorageError>;
    async fn bulk_assign_tag(
        &self,
        tag_id: i64,
        quick_command_ids: &[i64],
    ) -> Result<(), StorageError>;
}
