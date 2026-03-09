//! SSH 挂起/恢复/临时日志

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspendedSession {
    pub id: String,
    pub connection_id: i64,
    pub connection_name: String,
    pub suspended_at: String,
    /// 缓存的最近终端输出 (ring buffer)
    #[serde(skip)]
    pub output_buffer: Vec<u8>,
}

/// 挂起会话管理器
#[derive(Clone)]
pub struct SuspendManager {
    sessions: Arc<RwLock<HashMap<String, SuspendedSession>>>,
}

impl SuspendManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 挂起一个会话
    pub async fn suspend(
        &self,
        session_id: &str,
        connection_id: i64,
        connection_name: &str,
    ) -> Result<(), String> {
        let now = chrono_now();
        let entry = SuspendedSession {
            id: session_id.to_string(),
            connection_id,
            connection_name: connection_name.to_string(),
            suspended_at: now,
            output_buffer: Vec::new(),
        };
        self.sessions
            .write()
            .await
            .insert(session_id.to_string(), entry);
        Ok(())
    }

    /// 恢复一个挂起的会话，返回缓存的输出
    pub async fn resume(&self, session_id: &str) -> Result<Vec<u8>, String> {
        let entry = self
            .sessions
            .write()
            .await
            .remove(session_id)
            .ok_or_else(|| format!("挂起会话 {session_id} 不存在"))?;
        Ok(entry.output_buffer)
    }

    /// 列出所有挂起的会话
    pub async fn list(&self) -> Vec<SuspendedSession> {
        self.sessions.read().await.values().cloned().collect()
    }

    /// 终止一个挂起的会话
    pub async fn terminate(&self, session_id: &str) -> Result<(), String> {
        self.sessions
            .write()
            .await
            .remove(session_id)
            .ok_or_else(|| format!("挂起会话 {session_id} 不存在"))?;
        Ok(())
    }

    /// 向挂起会话的输出缓冲区追加数据（最多保留 64KB）
    pub async fn append_output(&self, session_id: &str, data: &[u8]) {
        if let Some(entry) = self.sessions.write().await.get_mut(session_id) {
            entry.output_buffer.extend_from_slice(data);
            const MAX_BUF: usize = 65536;
            if entry.output_buffer.len() > MAX_BUF {
                let drain = entry.output_buffer.len() - MAX_BUF;
                entry.output_buffer.drain(..drain);
            }
        }
    }
}

fn chrono_now() -> String {
    // 简单的 ISO 时间戳
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}
