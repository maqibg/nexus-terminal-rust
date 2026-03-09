//! Settings repository trait — implemented by storage-sqlite.

use crate::model::*;
use async_trait::async_trait;
use shared_utils::StorageError;

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    // Key-value settings
    async fn get_setting(&self, key: &str) -> Result<Option<String>, StorageError>;
    async fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError>;
    async fn get_all_settings(&self) -> Result<Vec<Setting>, StorageError>;

    // Appearance settings
    async fn get_appearance(&self, key: &str) -> Result<Option<String>, StorageError>;
    async fn set_appearance(&self, key: &str, value: &str) -> Result<(), StorageError>;
    async fn get_all_appearance(&self) -> Result<Vec<Setting>, StorageError>;

    // Terminal themes
    async fn list_themes(&self) -> Result<Vec<TerminalTheme>, StorageError>;
    async fn get_theme(&self, id: i64) -> Result<Option<TerminalTheme>, StorageError>;
    async fn create_theme(&self, theme: &TerminalTheme) -> Result<i64, StorageError>;
    async fn update_theme(&self, theme: &TerminalTheme) -> Result<bool, StorageError>;
    async fn delete_theme(&self, id: i64) -> Result<bool, StorageError>;

    // Notification channels
    async fn list_notification_channels(&self) -> Result<Vec<NotificationChannel>, StorageError>;
    async fn get_notification_channel(
        &self,
        id: i64,
    ) -> Result<Option<NotificationChannel>, StorageError>;
    async fn create_notification_channel(
        &self,
        channel: &NotificationChannel,
    ) -> Result<i64, StorageError>;
    async fn update_notification_channel(
        &self,
        channel: &NotificationChannel,
    ) -> Result<bool, StorageError>;
    async fn delete_notification_channel(&self, id: i64) -> Result<bool, StorageError>;
}
