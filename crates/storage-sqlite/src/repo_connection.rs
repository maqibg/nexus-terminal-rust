//! ConnectionRepository implementation for SQLite.

use async_trait::async_trait;
use connection_core::model::*;
use connection_core::repository::ConnectionRepository;
use sqlx::SqlitePool;

pub struct SqliteConnectionRepo {
    pool: SqlitePool,
}

impl SqliteConnectionRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn get_tags_for_connection(&self, conn_id: i64) -> Result<Vec<String>, String> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT t.name FROM tags t JOIN connection_tags ct ON t.id = ct.tag_id WHERE ct.connection_id = ?",
        )
        .bind(conn_id)
        .fetch_all(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(|(n,)| n).collect())
    }

    async fn sync_tags(&self, conn_id: i64, tags: &[String]) -> Result<(), String> {
        sqlx::query("DELETE FROM connection_tags WHERE connection_id = ?")
            .bind(conn_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        for tag_name in tags {
            // Upsert tag
            sqlx::query("INSERT OR IGNORE INTO tags (name) VALUES (?)")
                .bind(tag_name)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            sqlx::query(
                "INSERT INTO connection_tags (connection_id, tag_id) SELECT ?, id FROM tags WHERE name = ?",
            )
            .bind(conn_id).bind(tag_name)
            .execute(&self.pool).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

type ConnRow = (
    i64,
    String,
    String,
    String,
    i32,
    String,
    String,
    Option<String>,
    Option<i64>,
    Option<i64>,
    Option<String>,
    Option<String>,
    i32,
);

fn row_to_connection(r: ConnRow) -> Connection {
    Connection {
        id: r.0,
        name: r.1,
        conn_type: r.2,
        host: r.3,
        port: r.4,
        username: r.5,
        auth_method: r.6,
        encrypted_password: r.7,
        ssh_key_id: r.8,
        proxy_id: r.9,
        jump_chain: r.10,
        notes: r.11,
        sort_order: r.12,
        tags: vec![],
    }
}

const CONN_COLS: &str = "id, name, type, host, port, username, auth_method, encrypted_password, ssh_key_id, proxy_id, jump_chain, notes, sort_order";

#[async_trait]
impl ConnectionRepository for SqliteConnectionRepo {
    async fn list_connections(&self) -> Result<Vec<Connection>, String> {
        let rows: Vec<ConnRow> = sqlx::query_as(&format!(
            "SELECT {CONN_COLS} FROM connections ORDER BY sort_order, id"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        let mut conns: Vec<Connection> = rows.into_iter().map(row_to_connection).collect();
        for conn in &mut conns {
            conn.tags = self.get_tags_for_connection(conn.id).await?;
        }
        Ok(conns)
    }

    async fn get_connection(&self, id: i64) -> Result<Option<Connection>, String> {
        let row: Option<ConnRow> =
            sqlx::query_as(&format!("SELECT {CONN_COLS} FROM connections WHERE id = ?"))
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
        match row {
            None => Ok(None),
            Some(r) => {
                let mut conn = row_to_connection(r);
                conn.tags = self.get_tags_for_connection(conn.id).await?;
                Ok(Some(conn))
            }
        }
    }

    async fn create_connection(
        &self,
        input: &ConnectionInput,
        encrypted_password: Option<&str>,
    ) -> Result<i64, String> {
        let result = sqlx::query(
            "INSERT INTO connections (name, type, host, port, username, auth_method, encrypted_password, ssh_key_id, proxy_id, jump_chain, notes, sort_order) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
        )
         .bind(&input.name)
        .bind(input.conn_type.as_deref().unwrap_or("SSH"))
        .bind(&input.host)
        .bind(input.port.unwrap_or(22))
        .bind(input.username.as_deref().unwrap_or("root"))
        .bind(input.auth_method.as_deref().unwrap_or("password"))
        .bind(encrypted_password)
        .bind(input.ssh_key_id).bind(input.proxy_id)
        .bind(&input.jump_chain).bind(&input.notes)
        .bind(input.sort_order.unwrap_or(0))
        .execute(&self.pool).await.map_err(|e| e.to_string())?;
        let id = result.last_insert_rowid();
        if let Some(tags) = &input.tags {
            self.sync_tags(id, tags).await?;
        }
        Ok(id)
    }

    async fn update_connection(
        &self,
        id: i64,
        input: &ConnectionInput,
        encrypted_password: Option<&str>,
    ) -> Result<bool, String> {
        let result = sqlx::query(
            "UPDATE connections SET name=?, type=?, host=?, port=?, username=?, auth_method=?, encrypted_password=COALESCE(?,encrypted_password), ssh_key_id=?, proxy_id=?, jump_chain=?, notes=?, sort_order=?, updated_at=datetime('now') WHERE id=?",
        )
         .bind(&input.name)
        .bind(input.conn_type.as_deref().unwrap_or("SSH"))
        .bind(&input.host)
        .bind(input.port.unwrap_or(22))
        .bind(input.username.as_deref().unwrap_or("root"))
        .bind(input.auth_method.as_deref().unwrap_or("password"))
        .bind(encrypted_password)
        .bind(input.ssh_key_id).bind(input.proxy_id)
        .bind(&input.jump_chain).bind(&input.notes)
        .bind(input.sort_order.unwrap_or(0))
        .bind(id)
        .execute(&self.pool).await.map_err(|e| e.to_string())?;
        if let Some(tags) = &input.tags {
            self.sync_tags(id, tags).await?;
        }
        Ok(result.rows_affected() > 0)
    }

    async fn delete_connection(&self, id: i64) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM connections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(result.rows_affected() > 0)
    }

    async fn reorder_connections(&self, ids: &[i64]) -> Result<(), String> {
        for (i, id) in ids.iter().enumerate() {
            sqlx::query("UPDATE connections SET sort_order = ? WHERE id = ?")
                .bind(i as i32)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn list_tags(&self) -> Result<Vec<Tag>, String> {
        sqlx::query_as::<_, (i64, String)>("SELECT id, name FROM tags ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(id, name)| Tag { id, name })
                    .collect()
            })
            .map_err(|e| e.to_string())
    }

    async fn create_tag(&self, name: &str) -> Result<i64, String> {
        let result = sqlx::query("INSERT INTO tags (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(result.last_insert_rowid())
    }

    async fn delete_tag(&self, id: i64) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_ssh_keys(&self) -> Result<Vec<SshKey>, String> {
        sqlx::query_as::<_, (i64, String, String, Option<String>)>(
            "SELECT id, name, encrypted_private_key, encrypted_passphrase FROM ssh_keys ORDER BY id",
        )
        .fetch_all(&self.pool).await
        .map(|rows| rows.into_iter().map(|(id, name, encrypted_private_key, encrypted_passphrase)| SshKey {
            id, name, encrypted_private_key, encrypted_passphrase,
        }).collect())
        .map_err(|e| e.to_string())
    }

    async fn get_ssh_key(&self, id: i64) -> Result<Option<SshKey>, String> {
        sqlx::query_as::<_, (i64, String, String, Option<String>)>(
            "SELECT id, name, encrypted_private_key, encrypted_passphrase FROM ssh_keys WHERE id = ?",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map(|r| r.map(|(id, name, encrypted_private_key, encrypted_passphrase)| SshKey {
            id, name, encrypted_private_key, encrypted_passphrase,
        }))
        .map_err(|e| e.to_string())
    }

    async fn create_ssh_key(
        &self,
        name: &str,
        encrypted_private_key: &str,
        encrypted_passphrase: Option<&str>,
    ) -> Result<i64, String> {
        let result = sqlx::query("INSERT INTO ssh_keys (name, encrypted_private_key, encrypted_passphrase) VALUES (?,?,?)")
            .bind(name).bind(encrypted_private_key).bind(encrypted_passphrase)
            .execute(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(result.last_insert_rowid())
    }

    async fn update_ssh_key(
        &self,
        id: i64,
        name: &str,
        encrypted_private_key: Option<&str>,
        encrypted_passphrase: Option<&str>,
    ) -> Result<bool, String> {
        let result = sqlx::query(
            "UPDATE ssh_keys SET name=?, encrypted_private_key=COALESCE(?,encrypted_private_key), encrypted_passphrase=COALESCE(?,encrypted_passphrase), updated_at=datetime('now') WHERE id=?",
        )
        .bind(name).bind(encrypted_private_key).bind(encrypted_passphrase).bind(id)
        .execute(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_ssh_key(&self, id: i64) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM ssh_keys WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_proxies(&self) -> Result<Vec<Proxy>, String> {
        sqlx::query_as::<_, (i64, String, String, String, i32, Option<String>, String, Option<String>, Option<String>)>(
            "SELECT id, name, type, host, port, username, auth_method, encrypted_password, encrypted_private_key FROM proxies ORDER BY id",
        )
        .fetch_all(&self.pool).await
        .map(|rows| rows.into_iter().map(|(id, name, proxy_type, host, port, username, auth_method, encrypted_password, encrypted_private_key)| Proxy {
            id, name, proxy_type, host, port, username, auth_method, encrypted_password, encrypted_private_key,
        }).collect())
        .map_err(|e| e.to_string())
    }

    async fn get_proxy(&self, id: i64) -> Result<Option<Proxy>, String> {
        sqlx::query_as::<_, (i64, String, String, String, i32, Option<String>, String, Option<String>, Option<String>)>(
            "SELECT id, name, type, host, port, username, auth_method, encrypted_password, encrypted_private_key FROM proxies WHERE id = ?",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map(|r| r.map(|(id, name, proxy_type, host, port, username, auth_method, encrypted_password, encrypted_private_key)| Proxy {
            id, name, proxy_type, host, port, username, auth_method, encrypted_password, encrypted_private_key,
        }))
        .map_err(|e| e.to_string())
    }

    async fn create_proxy(&self, p: &Proxy) -> Result<i64, String> {
        let result = sqlx::query(
            "INSERT INTO proxies (name, type, host, port, username, auth_method, encrypted_password, encrypted_private_key) VALUES (?,?,?,?,?,?,?,?)",
        )
        .bind(&p.name).bind(&p.proxy_type).bind(&p.host).bind(p.port)
        .bind(&p.username).bind(&p.auth_method).bind(&p.encrypted_password).bind(&p.encrypted_private_key)
        .execute(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(result.last_insert_rowid())
    }

    async fn update_proxy(&self, p: &Proxy) -> Result<bool, String> {
        let result = sqlx::query(
            "UPDATE proxies SET name=?, type=?, host=?, port=?, username=?, auth_method=?, encrypted_password=?, encrypted_private_key=?, updated_at=datetime('now') WHERE id=?",
        )
        .bind(&p.name).bind(&p.proxy_type).bind(&p.host).bind(p.port)
        .bind(&p.username).bind(&p.auth_method).bind(&p.encrypted_password).bind(&p.encrypted_private_key)
        .bind(p.id)
        .execute(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_proxy(&self, id: i64) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM proxies WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(result.rows_affected() > 0)
    }
}
