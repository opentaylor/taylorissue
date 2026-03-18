pub mod commands;
pub mod config;
pub mod kernel;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Install
            commands::install::start_install,
            // Repair
            commands::repair::start_repair,
            commands::repair::fix_step,
            commands::repair::start_custom_fix,
            // Uninstall
            commands::uninstall::start_uninstall,
            // Message
            commands::message::list_agents,
            commands::message::message_chat,
            commands::message::get_conversation,
            commands::message::append_conversation,
            commands::message::clear_conversation,
            // Skill
            commands::skill::list_skills,
            commands::skill::install_skill,
            commands::skill::uninstall_skill,
            commands::skill::search_clawhub,
            commands::skill::install_clawhub_skill,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
