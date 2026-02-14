//! Settings repository trait — implemented by storage-sqlite.

use crate::model::*;
use async_trait::async_trait;

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    // Key-value settings
    async fn get_setting(&self, key: &str) -> Result<Option<String>, String>;
    async fn set_setting(&self, key: &str, value: &str) -> Result<(), String>;
    async fn get_all_settings(&self) -> Result<Vec<Setting>, String>;

    // Appearance settings
    async fn get_appearance(&self, key: &str) -> Result<Option<String>, String>;
    async fn set_appearance(&self, key: &str, value: &str) -> Result<(), String>;
    async fn get_all_appearance(&self) -> Result<Vec<Setting>, String>;

    // Terminal themes
    async fn list_themes(&self) -> Result<Vec<TerminalTheme>, String>;
    async fn get_theme(&self, id: i64) -> Result<Option<TerminalTheme>, String>;
    async fn create_theme(&self, theme: &TerminalTheme) -> Result<i64, String>;
    async fn update_theme(&self, theme: &TerminalTheme) -> Result<bool, String>;
    async fn delete_theme(&self, id: i64) -> Result<bool, String>;

    // Notification channels
    async fn list_notification_channels(&self) -> Result<Vec<NotificationChannel>, String>;
    async fn get_notification_channel(
        &self,
        id: i64,
    ) -> Result<Option<NotificationChannel>, String>;
    async fn create_notification_channel(
        &self,
        channel: &NotificationChannel,
    ) -> Result<i64, String>;
    async fn update_notification_channel(
        &self,
        channel: &NotificationChannel,
    ) -> Result<bool, String>;
    async fn delete_notification_channel(&self, id: i64) -> Result<bool, String>;
}
