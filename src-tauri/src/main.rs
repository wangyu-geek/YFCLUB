mod commands;
mod db;
mod models;
mod state;
mod utils;

use state::AppState;

fn main() {
    let state = AppState::initialize().expect("failed to initialize app state");
    db::initialize_database(&state).expect("failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::member_search,
            commands::member_get_detail,
            commands::member_create,
            commands::member_update,
            commands::member_disable,
            commands::consume_create,
            commands::gift_list,
            commands::gift_save,
            commands::gift_redeem,
            commands::settings_get,
            commands::settings_save,
            commands::report_dashboard,
            commands::backup_create,
            commands::backup_restore,
            commands::migration_precheck,
            commands::migration_execute,
            commands::migration_get_report,
            commands::migration_export,
            commands::operation_logs_query
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
