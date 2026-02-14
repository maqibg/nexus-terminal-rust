use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickCommand {
    pub id: i64,
    pub name: String,
    pub command: String,
    pub usage_count: i64,
    pub variables: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct QuickCommandInput {
    pub name: String,
    pub command: String,
    pub variables: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickCommandTag {
    pub id: i64,
    pub name: String,
}
