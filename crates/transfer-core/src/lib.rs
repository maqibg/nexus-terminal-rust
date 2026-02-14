//! 文件传输管理

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferKind {
    Upload,
    Download,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTask {
    pub id: String,
    pub kind: TransferKind,
    pub file_name: String,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub status: TransferStatus,
    pub error: Option<String>,
}

/// 传输管理器 — 管理上传/下载任务队列和进度追踪
#[derive(Clone)]
pub struct TransferManager {
    tasks: Arc<DashMap<String, TransferTask>>,
    cancel_tokens: Arc<DashMap<String, watch::Sender<bool>>>,
}

impl TransferManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
            cancel_tokens: Arc::new(DashMap::new()),
        }
    }

    /// 创建新的传输任务
    pub fn create_task(
        &self,
        kind: TransferKind,
        file_name: &str,
        total_bytes: u64,
    ) -> (String, watch::Receiver<bool>) {
        let id = uuid::Uuid::new_v4().to_string();
        let task = TransferTask {
            id: id.clone(),
            kind,
            file_name: file_name.to_string(),
            total_bytes,
            transferred_bytes: 0,
            status: TransferStatus::Pending,
            error: None,
        };
        self.tasks.insert(id.clone(), task);
        let (tx, rx) = watch::channel(false);
        self.cancel_tokens.insert(id.clone(), tx);
        (id, rx)
    }

    /// 更新传输进度
    pub fn update_progress(&self, id: &str, transferred: u64) {
        if let Some(mut task) = self.tasks.get_mut(id) {
            task.transferred_bytes = transferred;
            task.status = TransferStatus::InProgress;
        }
    }

    /// 标记任务完成
    pub fn complete(&self, id: &str) {
        if let Some(mut task) = self.tasks.get_mut(id) {
            task.transferred_bytes = task.total_bytes;
            task.status = TransferStatus::Completed;
        }
        self.cancel_tokens.remove(id);
    }

    /// 标记任务失败
    pub fn fail(&self, id: &str, error: &str) {
        if let Some(mut task) = self.tasks.get_mut(id) {
            task.status = TransferStatus::Failed;
            task.error = Some(error.to_string());
        }
        self.cancel_tokens.remove(id);
    }

    /// 取消任务
    pub fn cancel(&self, id: &str) -> Result<(), String> {
        if let Some(tx) = self.cancel_tokens.get(id) {
            let _ = tx.send(true);
        }
        if let Some(mut task) = self.tasks.get_mut(id) {
            task.status = TransferStatus::Cancelled;
        }
        self.cancel_tokens.remove(id);
        Ok(())
    }

    /// 获取单个任务
    pub fn get_task(&self, id: &str) -> Option<TransferTask> {
        self.tasks.get(id).map(|t| t.clone())
    }

    /// 列出所有任务
    pub fn list_tasks(&self) -> Vec<TransferTask> {
        self.tasks.iter().map(|e| e.value().clone()).collect()
    }

    /// 移除已完成/失败/取消的任务
    pub fn cleanup(&self) {
        self.tasks.retain(|_, t| {
            matches!(
                t.status,
                TransferStatus::Pending | TransferStatus::InProgress
            )
        });
    }
}
