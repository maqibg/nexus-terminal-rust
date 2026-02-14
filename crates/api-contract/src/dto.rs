//! Common DTO types shared across domains.

use serde::{Deserialize, Serialize};

/// Paginated response wrapper.
#[derive(Debug, Serialize, Deserialize)]
pub struct Paged<T> {
    pub items: Vec<T>,
    pub total: u64,
}

/// Simple boolean setting DTO.
#[derive(Debug, Serialize, Deserialize)]
pub struct BoolSettingDto {
    pub value: bool,
}

/// Simple boolean setting request.
#[derive(Debug, Serialize, Deserialize)]
pub struct BoolSettingReq {
    pub value: bool,
}
