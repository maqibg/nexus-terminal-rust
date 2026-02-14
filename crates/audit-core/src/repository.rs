use crate::model::AuditLog;
use async_trait::async_trait;

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn list_logs(&self, limit: i64, offset: i64) -> Result<Vec<AuditLog>, String>;
    async fn count_logs(&self) -> Result<i64, String>;
    async fn create_log(&self, action_type: &str, details: Option<&str>) -> Result<i64, String>;
    async fn clear_logs(&self) -> Result<(), String>;
}
