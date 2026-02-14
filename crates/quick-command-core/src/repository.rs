use crate::model::*;
use async_trait::async_trait;

#[async_trait]
pub trait QuickCommandRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<QuickCommand>, String>;
    async fn get(&self, id: i64) -> Result<Option<QuickCommand>, String>;
    async fn create(&self, input: &QuickCommandInput) -> Result<i64, String>;
    async fn update(&self, id: i64, input: &QuickCommandInput) -> Result<bool, String>;
    async fn delete(&self, id: i64) -> Result<bool, String>;
    async fn increment_usage(&self, id: i64) -> Result<(), String>;

    // Quick command tags
    async fn list_tags(&self) -> Result<Vec<QuickCommandTag>, String>;
    async fn create_tag(&self, name: &str) -> Result<i64, String>;
    async fn delete_tag(&self, id: i64) -> Result<bool, String>;
    async fn bulk_assign_tag(&self, tag_id: i64, quick_command_ids: &[i64]) -> Result<(), String>;
}
