//! Nexus Terminal - Tauri 2 desktop application entry point

mod commands;
mod state;
mod status_monitor;

use anyhow::{anyhow, Context};
use log::LevelFilter;
use state::{AppState, RuntimePaths};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Info)
                .level_for("nexus_terminal", LevelFilter::Info)
                .level_for("sqlx", LevelFilter::Warn)
                .level_for("russh", LevelFilter::Warn)
                .level_for("russh_sftp", LevelFilter::Warn)
                .level_for("tao", LevelFilter::Error)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(
            |app: &mut tauri::App| -> Result<(), Box<dyn std::error::Error>> {
                let executable_path =
                    std::env::current_exe().context("failed to get current executable path")?;
                let executable_dir = executable_path
                    .parent()
                    .ok_or_else(|| anyhow!("failed to get executable directory"))?
                    .to_path_buf();
                let runtime_paths = RuntimePaths::new(executable_dir);
                runtime_paths.ensure_dirs().map_err(anyhow::Error::msg)?;

                let db_path = runtime_paths.data_dir.join("nexus.db");
                let key_path = runtime_paths.data_dir.join(".encryption_key");

                // Init or load encryption key
                let crypto = if key_path.exists() {
                    let hex_key = std::fs::read_to_string(&key_path).with_context(|| {
                        format!("failed to read encryption key: {}", key_path.display())
                    })?;
                    shared_utils::crypto::CryptoService::from_hex_key(hex_key.trim())
                        .map_err(|error| anyhow!("invalid encryption key: {error}"))?
                } else {
                    let hex_key = shared_utils::crypto::CryptoService::generate_key_hex();
                    std::fs::write(&key_path, &hex_key).with_context(|| {
                        format!("failed to write encryption key: {}", key_path.display())
                    })?;
                    shared_utils::crypto::CryptoService::from_hex_key(&hex_key)
                        .map_err(|error| anyhow!("invalid generated encryption key: {error}"))?
                };

                let storage = tauri::async_runtime::block_on(async {
                    storage_sqlite::init_pool(&db_path)
                        .await
                        .map_err(|error| anyhow!("failed to init database: {error}"))
                })?;

                let app_state = AppState::new(storage, crypto, runtime_paths);
                tauri::async_runtime::block_on(app_state.init_auth_state());

                app.manage(app_state);
                Ok(())
            },
        )
        .invoke_handler(tauri::generate_handler![
            // Status
            commands::status::get_backend_health,
            commands::status::get_connection_runtime_status,
            commands::status::get_runtime_paths,
            commands::status::set_status_monitor_enabled,
            // Auth
            commands::auth::auth_status,
            commands::auth::auth_setup,
            commands::auth::auth_login,
            commands::auth::auth_verify_2fa,
            commands::auth::auth_logout,
            commands::auth::auth_change_password,
            commands::auth::auth_setup_2fa,
            commands::auth::auth_verify_activate_2fa,
            commands::auth::auth_disable_2fa,
            commands::auth::passkey_list,
            commands::auth::passkey_register_start,
            commands::auth::passkey_register_finish,
            commands::auth::passkey_delete,
            commands::auth::passkey_rename,
            // Connections
            commands::connections::connection_list,
            commands::connections::connection_get,
            commands::connections::connection_create,
            commands::connections::connection_update,
            commands::connections::connection_delete,
            commands::connections::connection_reorder,
            commands::connections::tag_list,
            commands::connections::tag_create,
            commands::connections::tag_delete,
            commands::connections::ssh_key_list,
            commands::connections::ssh_key_delete,
            commands::connections::ssh_key_create,
            commands::connections::ssh_key_update,
            commands::connections::proxy_list,
            commands::connections::proxy_delete,
            commands::connections::proxy_create,
            commands::connections::proxy_update,
            commands::connections::connection_test,
            commands::connections::connection_test_unsaved,
            commands::connections::connection_clone,
            commands::connections::connection_export,
            commands::connections::connection_export_to_file,
            commands::connections::connection_import,
            commands::connections::app_export,
            commands::connections::app_export_to_file,
            commands::connections::app_import,
            commands::connections::app_import_from_file,
            commands::connections::app_reset_data,
            commands::connections::app_reset_data_counts,
            // Settings
            commands::settings::settings_get_all,
            commands::settings::settings_set,
            commands::settings::appearance_get_all,
            commands::settings::appearance_set,
            commands::settings::theme_list,
            commands::settings::theme_get,
            commands::settings::theme_create,
            commands::settings::theme_update,
            commands::settings::theme_delete,
            commands::settings::notification_channel_list,
            commands::settings::notification_channel_create,
            commands::settings::notification_channel_update,
            commands::settings::notification_channel_delete,
            // AI
            commands::ai::ai_get_all_channels,
            commands::ai::ai_add_channel,
            commands::ai::ai_update_channel,
            commands::ai::ai_delete_channel,
            commands::ai::ai_verify_channel,
            commands::ai::ai_fetch_models,
            commands::ai::ai_add_model,
            commands::ai::ai_delete_model,
            commands::ai::ai_get_all_models,
            commands::ai::ai_set_default_model,
            commands::ai::ai_get_config,
            commands::ai::ai_update_config,
            commands::ai::ai_request,
            commands::ai::ai_request_with_model,
            commands::ai::ai_cancel_request,
            commands::ai::ai_get_chat_history,
            commands::ai::ai_save_chat_history,
            commands::ai::ai_clear_chat_history,
            commands::ai::ai_get_terminal_chat_history,
            commands::ai::ai_save_terminal_chat_history,
            commands::ai::ai_clear_terminal_chat_history,
            // Auxiliary
            commands::auxiliary::audit_log_list,
            commands::auxiliary::audit_log_count,
            commands::auxiliary::audit_log_clear,
            commands::auxiliary::command_history_list,
            commands::auxiliary::command_history_clear,
            commands::auxiliary::command_history_delete,
            commands::auxiliary::command_history_add,
            commands::auxiliary::path_history_list,
            commands::auxiliary::path_history_add,
            commands::auxiliary::path_history_clear,
            commands::auxiliary::favorite_path_list,
            commands::auxiliary::favorite_path_create,
            commands::auxiliary::favorite_path_delete,
            commands::auxiliary::favorite_path_mark_used,
            commands::auxiliary::favorite_path_update,
            commands::auxiliary::quick_command_list,
            commands::auxiliary::quick_command_get,
            commands::auxiliary::quick_command_create,
            commands::auxiliary::quick_command_update,
            commands::auxiliary::quick_command_delete,
            commands::auxiliary::quick_command_use,
            commands::auxiliary::quick_command_tag_list,
            commands::auxiliary::quick_command_tag_create,
            commands::auxiliary::quick_command_tag_delete,
            commands::auxiliary::quick_command_bulk_assign_tag,
            // SSH
            commands::ssh::ssh_connect,
            commands::ssh::ssh_write,
            commands::ssh::ssh_resize,
            commands::ssh::ssh_close,
            commands::ssh::ssh_session_list,
            commands::ssh::ssh_exec_command,
            commands::ssh::ssh_take_output_backlog,
            commands::ssh::ssh_accept_host_key,
            commands::ssh::ssh_host_key_list,
            commands::ssh::ssh_host_key_delete,
            commands::ssh::ssh_host_key_get,
            commands::ssh_suspend::ssh_suspend_list,
            commands::ssh_suspend::ssh_suspend,
            commands::ssh_suspend::ssh_resume,
            commands::ssh_suspend::ssh_suspend_terminate,
            // SFTP
            commands::sftp::sftp_open,
            commands::sftp::sftp_open_override,
            commands::sftp::sftp_close,
            commands::sftp::sftp_list_dir,
            commands::sftp::sftp_read_file,
            commands::sftp::sftp_write_file,
            commands::sftp::sftp_mkdir,
            commands::sftp::sftp_rmdir,
            commands::sftp::sftp_remove_file,
            commands::sftp::sftp_rename,
            commands::sftp::sftp_copy_entry,
            commands::sftp::sftp_stat,
            commands::sftp::sftp_chmod,
            commands::sftp::sftp_upload_chunk,
            commands::sftp::sftp_download_file,
            commands::sftp::sftp_download_to_disk,
            commands::sftp::sftp_upload_from_disk,
            commands::sftp::sftp_upload_entry_from_disk,
            commands::sftp::sftp_collect_local_upload_entries,
            commands::sftp::sftp_cancel_task,
            commands::sftp::sftp_download_directory_to_disk,
            // Transfer
            commands::transfer::transfer_send,
            commands::transfer::transfer_list,
            commands::transfer::transfer_get,
            commands::transfer::transfer_cancel,
            commands::transfer::transfer_pause,
            commands::transfer::transfer_resume,
            commands::transfer::transfer_pause_all,
            commands::transfer::transfer_resume_all,
            commands::transfer::transfer_cancel_all,
            commands::transfer::transfer_cleanup_completed,
            // Desktop
            commands::desktop::desktop_open_rdp,
            commands::desktop::desktop_open_rdp_connection,
            commands::desktop::desktop_open_vnc,
            commands::desktop::desktop_open_vnc_connection,
            commands::desktop::desktop_rdp_status,
            commands::desktop::desktop_rdp_list_sessions,
            commands::desktop::desktop_rdp_disconnect_connection,
            commands::desktop::desktop_vnc_status,
            commands::desktop::desktop_vnc_list_sessions,
            commands::desktop::desktop_vnc_disconnect,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            tracing::error!("error while running tauri application: {error}");
        });
}
