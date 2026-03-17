use tauri::{
    image::Image,
    menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;

use crate::commands::{load_app_config, save_app_config};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_hide = MenuItemBuilder::with_id("show_hide", "Show Window").build(app)?;
    let separator1 = PredefinedMenuItem::separator(app)?;

    let notifications_enabled = load_app_config(app).notifications_enabled;
    let notifications = CheckMenuItemBuilder::with_id("notifications", "Notifications")
        .checked(notifications_enabled)
        .build(app)?;

    let auto_start_enabled = load_app_config(app).auto_start;
    let auto_start = CheckMenuItemBuilder::with_id("auto_start", "Start on Login")
        .checked(auto_start_enabled)
        .build(app)?;

    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_hide)
        .item(&separator1)
        .item(&notifications)
        .item(&auto_start)
        .item(&separator2)
        .item(&quit)
        .build()?;

    let icon = app.default_window_icon().cloned().unwrap_or_else(|| {
        Image::new_owned(vec![0u8; 4 * 32 * 32], 32, 32)
    });

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("Agent Playground")
        .menu(&menu)
        .on_menu_event(move |app, event| {
            let event_id = event.id().0.as_str();
            match event_id {
                "show_hide" => {
                    toggle_window(app);
                }
                "notifications" => {
                    let mut config = load_app_config(app);
                    config.notifications_enabled = !config.notifications_enabled;
                    save_app_config(app, &config);
                }
                "auto_start" => {
                    let mut config = load_app_config(app);
                    config.auto_start = !config.auto_start;
                    save_app_config(app, &config);

                    let autostart = app.autolaunch();
                    if config.auto_start {
                        let _ = autostart.enable();
                    } else {
                        let _ = autostart.disable();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray_icon, event| {
            if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                toggle_window(tray_icon.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
