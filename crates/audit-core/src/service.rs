//! Convenience service for audit logging.

use crate::repository::AuditRepository;
use std::sync::Arc;

/// Thin wrapper providing fire-and-forget audit logging.
#[derive(Clone)]
pub struct AuditService {
    repo: Arc<dyn AuditRepository>,
}

impl AuditService {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }

    /// Log an action. Errors are silently ignored (audit must not block business logic).
    pub async fn log(&self, action: &str, details: Option<&str>) {
        let _ = self.repo.create_log(action, details).await;
    }

    /// Spawn a background audit log (non-blocking).
    pub fn log_async(&self, action: String, details: Option<String>) {
        let svc = self.clone();
        tokio::spawn(async move {
            svc.log(&action, details.as_deref()).await;
        });
    }
}
