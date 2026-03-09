//! 文件传输管理

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::watch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferKind {
    Upload,
    Download,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Paused,
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
    pause_flags: Arc<DashMap<String, Arc<AtomicBool>>>,
}

impl TransferManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
            cancel_tokens: Arc::new(DashMap::new()),
            pause_flags: Arc::new(DashMap::new()),
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
        self.pause_flags
            .insert(id.clone(), Arc::new(AtomicBool::new(false)));
        (id, rx)
    }

    /// 更新传输进度
    pub fn update_progress(&self, id: &str, transferred: u64) {
        if let Some(mut task) = self.tasks.get_mut(id) {
            task.transferred_bytes = transferred;
            if task.status != TransferStatus::Paused {
                task.status = TransferStatus::InProgress;
            }
        }
    }

    pub fn pause(&self, id: &str) -> Result<(), String> {
        if !self.tasks.contains_key(id) {
            return Err(format!("task '{}' not found", id));
        }
        if let Some(flag) = self.pause_flags.get(id) {
            flag.store(true, Ordering::Relaxed);
        }
        if let Some(mut task) = self.tasks.get_mut(id) {
            if matches!(task.status, TransferStatus::Pending | TransferStatus::InProgress) {
                task.status = TransferStatus::Paused;
            }
        }
        Ok(())
    }

    pub fn resume(&self, id: &str) -> Result<(), String> {
        if !self.tasks.contains_key(id) {
            return Err(format!("task '{}' not found", id));
        }
        if let Some(flag) = self.pause_flags.get(id) {
            flag.store(false, Ordering::Relaxed);
        }
        if let Some(mut task) = self.tasks.get_mut(id) {
            if task.status == TransferStatus::Paused {
                task.status = if task.transferred_bytes > 0 {
                    TransferStatus::InProgress
                } else {
                    TransferStatus::Pending
                };
            }
        }
        Ok(())
    }

    pub fn pause_all(&self) {
        let task_ids: Vec<String> = self.tasks.iter().map(|entry| entry.key().clone()).collect();
        for id in task_ids {
            let _ = self.pause(&id);
        }
    }

    pub fn resume_all(&self) {
        let task_ids: Vec<String> = self.tasks.iter().map(|entry| entry.key().clone()).collect();
        for id in task_ids {
            let _ = self.resume(&id);
        }
    }

    /// 标记任务完成
    pub fn complete(&self, id: &str) {
        if let Some(mut task) = self.tasks.get_mut(id) {
            task.transferred_bytes = task.total_bytes;
            task.status = TransferStatus::Completed;
        }
        self.cancel_tokens.remove(id);
        self.pause_flags.remove(id);
    }

    /// 标记任务失败
    pub fn fail(&self, id: &str, error: &str) {
        if let Some(mut task) = self.tasks.get_mut(id) {
            task.status = TransferStatus::Failed;
            task.error = Some(error.to_string());
        }
        self.cancel_tokens.remove(id);
        self.pause_flags.remove(id);
    }

    /// 取消任务
    pub fn cancel(&self, id: &str) -> Result<(), String> {
        if !self.tasks.contains_key(id) {
            return Err(format!("task '{}' not found", id));
        }
        if let Some(tx) = self.cancel_tokens.get(id) {
            let _ = tx.send(true);
        }
        if let Some(mut task) = self.tasks.get_mut(id) {
            task.status = TransferStatus::Cancelled;
        }
        self.cancel_tokens.remove(id);
        self.pause_flags.remove(id);
        Ok(())
    }

    pub fn cancel_all(&self) {
        let task_ids: Vec<String> = self
            .tasks
            .iter()
            .filter(|entry| {
                matches!(
                    entry.value().status,
                    TransferStatus::Pending | TransferStatus::InProgress | TransferStatus::Paused
                )
            })
            .map(|entry| entry.key().clone())
            .collect();
        for task_id in task_ids {
            let _ = self.cancel(&task_id);
        }
    }

    pub fn is_paused(&self, id: &str) -> bool {
        self.pause_flags
            .get(id)
            .map(|flag| flag.load(Ordering::Relaxed))
            .unwrap_or(false)
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
                TransferStatus::Pending | TransferStatus::InProgress | TransferStatus::Paused
            )
        });
    }
}
