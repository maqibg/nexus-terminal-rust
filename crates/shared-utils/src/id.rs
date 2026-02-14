use uuid::Uuid;

/// Generate a new UUID v4 string
pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}
