//! SettingsRepository implementation for SQLite.

use async_trait::async_trait;
use settings_core::model::*;
use settings_core::repository::SettingsRepository;
use sqlx::{Row, SqlitePool};

fn row_to_theme(r: &sqlx::sqlite::SqliteRow) -> TerminalTheme {
    TerminalTheme {
        id: r.get("id"),
        name: r.get("name"),
        theme_type: r.get("theme_type"),
        background: r.get("background"),
        foreground: r.get("foreground"),
        cursor: r.get("cursor"),
        cursor_accent: r.get("cursor_accent"),
        selection_background: r.get("selection_background"),
        selection_foreground: r.get("selection_foreground"),
        selection_inactive_background: r.get("selection_inactive_background"),
        black: r.get("black"),
        red: r.get("red"),
        green: r.get("green"),
        yellow: r.get("yellow"),
        blue: r.get("blue"),
        magenta: r.get("magenta"),
        cyan: r.get("cyan"),
        white: r.get("white"),
        bright_black: r.get("bright_black"),
        bright_red: r.get("bright_red"),
        bright_green: r.get("bright_green"),
        bright_yellow: r.get("bright_yellow"),
        bright_blue: r.get("bright_blue"),
        bright_magenta: r.get("bright_magenta"),
        bright_cyan: r.get("bright_cyan"),
        bright_white: r.get("bright_white"),
    }
}

pub struct SqliteSettingsRepo {
    pool: SqlitePool,
}

impl SqliteSettingsRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SettingsRepository for SqliteSettingsRepo {
    async fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map(|r| r.map(|(v,)| v))
            .map_err(|e| e.to_string())
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
            .bind(key).bind(value)
            .execute(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_all_settings(&self) -> Result<Vec<Setting>, String> {
        sqlx::query_as::<_, (String, String)>("SELECT key, value FROM settings")
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(key, value)| Setting { key, value })
                    .collect()
            })
            .map_err(|e| e.to_string())
    }

    async fn get_appearance(&self, key: &str) -> Result<Option<String>, String> {
        sqlx::query_as::<_, (String,)>("SELECT value FROM appearance_settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map(|r| r.map(|(v,)| v))
            .map_err(|e| e.to_string())
    }

    async fn set_appearance(&self, key: &str, value: &str) -> Result<(), String> {
        sqlx::query("INSERT INTO appearance_settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
            .bind(key).bind(value)
            .execute(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_all_appearance(&self) -> Result<Vec<Setting>, String> {
        sqlx::query_as::<_, (String, String)>("SELECT key, value FROM appearance_settings")
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(key, value)| Setting { key, value })
                    .collect()
            })
            .map_err(|e| e.to_string())
    }

    async fn list_themes(&self) -> Result<Vec<TerminalTheme>, String> {
        let rows = sqlx::query(
            "SELECT id, name, theme_type, background, foreground, cursor, cursor_accent, selection_background, selection_foreground, selection_inactive_background, black, red, green, yellow, blue, magenta, cyan, white, bright_black, bright_red, bright_green, bright_yellow, bright_blue, bright_magenta, bright_cyan, bright_white FROM terminal_themes ORDER BY id",
        )
        .fetch_all(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(rows.iter().map(row_to_theme).collect())
    }

    async fn get_theme(&self, id: i64) -> Result<Option<TerminalTheme>, String> {
        let row = sqlx::query(
            "SELECT id, name, theme_type, background, foreground, cursor, cursor_accent, selection_background, selection_foreground, selection_inactive_background, black, red, green, yellow, blue, magenta, cyan, white, bright_black, bright_red, bright_green, bright_yellow, bright_blue, bright_magenta, bright_cyan, bright_white FROM terminal_themes WHERE id = ?",
        )
        .bind(id).fetch_optional(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(row.as_ref().map(row_to_theme))
    }

    async fn create_theme(&self, t: &TerminalTheme) -> Result<i64, String> {
        let result = sqlx::query(
            "INSERT INTO terminal_themes (name, theme_type, background, foreground, cursor, cursor_accent, selection_background, selection_foreground, selection_inactive_background, black, red, green, yellow, blue, magenta, cyan, white, bright_black, bright_red, bright_green, bright_yellow, bright_blue, bright_magenta, bright_cyan, bright_white) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(&t.name).bind(&t.theme_type).bind(&t.background).bind(&t.foreground)
        .bind(&t.cursor).bind(&t.cursor_accent).bind(&t.selection_background).bind(&t.selection_foreground)
        .bind(&t.selection_inactive_background).bind(&t.black).bind(&t.red).bind(&t.green)
        .bind(&t.yellow).bind(&t.blue).bind(&t.magenta).bind(&t.cyan).bind(&t.white)
        .bind(&t.bright_black).bind(&t.bright_red).bind(&t.bright_green).bind(&t.bright_yellow)
        .bind(&t.bright_blue).bind(&t.bright_magenta).bind(&t.bright_cyan).bind(&t.bright_white)
        .execute(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(result.last_insert_rowid())
    }

    async fn update_theme(&self, t: &TerminalTheme) -> Result<bool, String> {
        let result = sqlx::query(
            "UPDATE terminal_themes SET name=?, theme_type=?, background=?, foreground=?, cursor=?, cursor_accent=?, selection_background=?, selection_foreground=?, selection_inactive_background=?, black=?, red=?, green=?, yellow=?, blue=?, magenta=?, cyan=?, white=?, bright_black=?, bright_red=?, bright_green=?, bright_yellow=?, bright_blue=?, bright_magenta=?, bright_cyan=?, bright_white=?, updated_at=datetime('now') WHERE id=?",
        )
        .bind(&t.name).bind(&t.theme_type).bind(&t.background).bind(&t.foreground)
        .bind(&t.cursor).bind(&t.cursor_accent).bind(&t.selection_background).bind(&t.selection_foreground)
        .bind(&t.selection_inactive_background).bind(&t.black).bind(&t.red).bind(&t.green)
        .bind(&t.yellow).bind(&t.blue).bind(&t.magenta).bind(&t.cyan).bind(&t.white)
        .bind(&t.bright_black).bind(&t.bright_red).bind(&t.bright_green).bind(&t.bright_yellow)
        .bind(&t.bright_blue).bind(&t.bright_magenta).bind(&t.bright_cyan).bind(&t.bright_white)
        .bind(t.id)
        .execute(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_theme(&self, id: i64) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM terminal_themes WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_notification_channels(&self) -> Result<Vec<NotificationChannel>, String> {
        sqlx::query_as::<_, (i64, String, String, bool, String, String)>(
            "SELECT id, channel_type, name, enabled != 0, config, enabled_events FROM notification_settings ORDER BY id",
        )
        .fetch_all(&self.pool).await
        .map(|rows| rows.into_iter().map(|(id, channel_type, name, enabled, config, enabled_events)| NotificationChannel {
            id, channel_type, name, enabled, config, enabled_events,
        }).collect())
        .map_err(|e| e.to_string())
    }

    async fn get_notification_channel(
        &self,
        id: i64,
    ) -> Result<Option<NotificationChannel>, String> {
        sqlx::query_as::<_, (i64, String, String, bool, String, String)>(
            "SELECT id, channel_type, name, enabled != 0, config, enabled_events FROM notification_settings WHERE id = ?",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map(|r| r.map(|(id, channel_type, name, enabled, config, enabled_events)| NotificationChannel {
            id, channel_type, name, enabled, config, enabled_events,
        }))
        .map_err(|e| e.to_string())
    }

    async fn create_notification_channel(&self, c: &NotificationChannel) -> Result<i64, String> {
        let result = sqlx::query(
            "INSERT INTO notification_settings (channel_type, name, enabled, config, enabled_events) VALUES (?,?,?,?,?)",
        )
        .bind(&c.channel_type).bind(&c.name).bind(c.enabled).bind(&c.config).bind(&c.enabled_events)
        .execute(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(result.last_insert_rowid())
    }

    async fn update_notification_channel(&self, c: &NotificationChannel) -> Result<bool, String> {
        let result = sqlx::query(
            "UPDATE notification_settings SET channel_type=?, name=?, enabled=?, config=?, enabled_events=?, updated_at=datetime('now') WHERE id=?",
        )
        .bind(&c.channel_type).bind(&c.name).bind(c.enabled).bind(&c.config).bind(&c.enabled_events).bind(c.id)
        .execute(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_notification_channel(&self, id: i64) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM notification_settings WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(result.rows_affected() > 0)
    }
}
