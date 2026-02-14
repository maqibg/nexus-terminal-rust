use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    pub id: i64,
    pub command: String,
    pub session_id: Option<String>,
    pub connection_id: Option<i64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathHistory {
    pub id: i64,
    pub path: String,
    pub connection_id: Option<i64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoritePath {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub connection_id: Option<i64>,
    pub last_used_at: Option<String>,
}
