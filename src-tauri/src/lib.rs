pub mod commands;
pub mod config;
pub mod kernel;
pub mod prompts;
pub mod services;

#[cfg(target_os = "windows")]
fn disable_webview2_overscroll(app: &tauri::App) {
    use tauri::Manager;

    let Some(main_window) = app.get_webview_window("main") else {
        return;
    };
    let _ = main_window.with_webview(|webview| unsafe {
        use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings6;
        use windows::core::Interface;

        let core = webview.controller().CoreWebView2().unwrap();
        let settings = core.Settings().unwrap();
        if let Ok(settings6) = settings.cast::<ICoreWebView2Settings6>() {
            let _ = settings6.SetIsSwipeNavigationEnabled(false);
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            #[cfg(target_os = "windows")]
            disable_webview2_overscroll(_app);
            Ok(())
        })
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
