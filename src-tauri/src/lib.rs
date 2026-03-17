mod commands;
mod tray;

use commands::{load_app_config, load_window_state, save_window_state, AppState, WindowState};
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

const BRIDGE_SCRIPT: &str = include_str!("../../src/bridge.js");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_deep_link::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::notify_new_message,
            commands::update_badge_count,
            commands::report_user_active,
            commands::get_app_config,
            commands::set_app_config,
        ])
        .on_page_load(|webview, _payload| {
            if let Err(err) = webview.eval(BRIDGE_SCRIPT) {
                eprintln!("Failed to inject bridge script: {}", err);
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();

            // Setup system tray
            tray::setup_tray(&handle)?;

            // Restore window state from store
            let window_state = load_window_state(&handle);
            if let Some(window) = app.get_webview_window("main") {
                if let (Some(pos_x), Some(pos_y)) = (window_state.x, window_state.y) {
                    let _ = window.set_position(tauri::LogicalPosition::new(pos_x, pos_y));
                }
                let _ = window.set_size(tauri::LogicalSize::new(
                    window_state.width,
                    window_state.height,
                ));
                if window_state.maximized {
                    let _ = window.maximize();
                }
            }

            // Navigate to web app URL (compile-time default, runtime override)
            let compile_time_url = option_env!("AGENT_PLAYGROUND_URL")
                .unwrap_or("http://localhost:3000");
            let web_url = std::env::var("AGENT_PLAYGROUND_URL")
                .unwrap_or_else(|_| compile_time_url.to_string());

            let config = load_app_config(&handle);
            let url = if config.web_app_url.is_empty() {
                web_url
            } else {
                config.web_app_url.clone()
            };

            if let Some(window) = app.get_webview_window("main") {
                // Navigate to remote URL (bridge injected via on_page_load)
                if let Ok(parsed_url) = url::Url::parse(&url) {
                    if let Err(err) = window.navigate(parsed_url) {
                        eprintln!("Failed to navigate to {}: {}", url, err);
                    }
                }

                // Handle window events: close → minimize, save state on move/resize
                let app_handle = handle.clone();
                window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            api.prevent_close();
                            if let Some(win) = app_handle.get_webview_window("main") {
                                save_current_window_state(&app_handle, &win);
                                let _ = win.hide();
                            }
                        }
                        tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
                            if let Some(win) = app_handle.get_webview_window("main") {
                                save_current_window_state(&app_handle, &win);
                            }
                        }
                        _ => {}
                    }
                });
            }

            // Register global shortcut
            let shortcut_str = config.global_shortcut.clone();
            if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                let shortcut_handle = handle.clone();
                let _ = app.global_shortcut().on_shortcut(
                    shortcut,
                    move |_app, _shortcut, _event| {
                        tray::toggle_window(&shortcut_handle);
                    },
                );
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn save_current_window_state(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    let position = window.outer_position().ok();
    let size = window.outer_size().ok();
    let maximized = window.is_maximized().unwrap_or(false);
    let scale = window.scale_factor().unwrap_or(1.0);

    let state = WindowState {
        x: position.map(|pos| pos.x as f64 / scale),
        y: position.map(|pos| pos.y as f64 / scale),
        width: size.map(|sz| sz.width as f64 / scale).unwrap_or(1200.0),
        height: size.map(|sz| sz.height as f64 / scale).unwrap_or(800.0),
        maximized,
    };

    save_window_state(app, &state);
}
