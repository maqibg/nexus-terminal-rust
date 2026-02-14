//! Nexus Terminal - Tauri 2 desktop application entry point

mod commands;
mod state;
mod status_monitor;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).ok();
            let db_path = app_dir.join("nexus.db");
            let key_path = app_dir.join(".encryption_key");

            // Init or load encryption key
            let crypto = if key_path.exists() {
                let hex_key =
                    std::fs::read_to_string(&key_path).expect("failed to read encryption key");
                shared_utils::crypto::CryptoService::from_hex_key(hex_key.trim())
                    .expect("invalid encryption key")
            } else {
                let hex_key = shared_utils::crypto::CryptoService::generate_key_hex();
                std::fs::write(&key_path, &hex_key).expect("failed to write encryption key");
                shared_utils::crypto::CryptoService::from_hex_key(&hex_key)
                    .expect("invalid encryption key")
            };

            let storage = tauri::async_runtime::block_on(async {
                storage_sqlite::init_pool(&db_path)
                    .await
                    .expect("failed to init database")
            });

            let app_state = AppState::new(storage, crypto);
            tauri::async_runtime::block_on(app_state.init_auth_state());

            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Status
            commands::status::get_backend_health,
            commands::status::get_connection_runtime_status,
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
            commands::connections::connection_import,
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
            // Auxiliary
            commands::auxiliary::audit_log_list,
            commands::auxiliary::audit_log_count,
            commands::auxiliary::audit_log_clear,
            commands::auxiliary::command_history_list,
            commands::auxiliary::command_history_clear,
            commands::auxiliary::command_history_add,
            commands::auxiliary::path_history_list,
            commands::auxiliary::path_history_add,
            commands::auxiliary::path_history_clear,
            commands::auxiliary::favorite_path_list,
            commands::auxiliary::favorite_path_create,
            commands::auxiliary::favorite_path_delete,
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
            // SFTP
            commands::sftp::sftp_open,
            commands::sftp::sftp_close,
            commands::sftp::sftp_list_dir,
            commands::sftp::sftp_read_file,
            commands::sftp::sftp_write_file,
            commands::sftp::sftp_mkdir,
            commands::sftp::sftp_rmdir,
            commands::sftp::sftp_remove_file,
            commands::sftp::sftp_rename,
            commands::sftp::sftp_stat,
            commands::sftp::sftp_chmod,
            commands::sftp::sftp_upload_chunk,
            commands::sftp::sftp_download_file,
            commands::sftp::sftp_download_to_disk,
            commands::sftp::sftp_upload_from_disk,
            commands::sftp::sftp_cancel_task,
            commands::sftp::sftp_download_directory_to_disk,
            // Transfer
            commands::transfer::transfer_send,
            commands::transfer::transfer_list,
            commands::transfer::transfer_get,
            commands::transfer::transfer_cancel,
            // Desktop
            commands::desktop::desktop_open_rdp,
            commands::desktop::desktop_open_rdp_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
