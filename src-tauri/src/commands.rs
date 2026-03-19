use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub notifications_enabled: bool,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub global_shortcut: String,
    pub web_app_url: String,
    pub check_updates: bool,
    pub last_update_check: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            notifications_enabled: true,
            auto_start: false,
            minimize_to_tray: true,
            global_shortcut: "CmdOrCtrl+Shift+A".to_string(),
            web_app_url: String::new(),
            check_updates: true,
            last_update_check: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: f64,
    pub height: f64,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: 1200.0,
            height: 800.0,
            maximized: false,
        }
    }
}

pub struct AppState {
    pub active_conversation_id: Mutex<Option<String>>,
    pub unread_count: Mutex<u32>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_conversation_id: Mutex::new(None),
            unread_count: Mutex::new(0),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NotifyNewMessagePayload {
    pub sender_name: String,
    pub message_text: String,
    pub conversation_id: String,
    pub conversation_name: Option<String>,
    pub is_group: bool,
    #[serde(default)]
    pub force: bool,
}

#[tauri::command]
pub fn notify_new_message(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: NotifyNewMessagePayload,
) {
    let config = load_app_config(&app);
    if !config.notifications_enabled {
        return;
    }

    if !payload.force {
        if let Some(window) = app.get_webview_window("main") {
            if window.is_focused().unwrap_or(false) {
                let active_id = state
                    .active_conversation_id
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if active_id.as_deref() == Some(&payload.conversation_id) {
                    return;
                }
            }
        }
    }

    let body = if payload.message_text.chars().count() > 100 {
        let truncated: String = payload.message_text.chars().take(100).collect();
        format!("{}...", truncated)
    } else {
        payload.message_text.clone()
    };

    let title = if payload.is_group {
        if let Some(ref group_name) = payload.conversation_name {
            format!("{} in {}", payload.sender_name, group_name)
        } else {
            payload.sender_name.clone()
        }
    } else {
        payload.sender_name.clone()
    };

    static NOTIFICATION_ID: AtomicI32 = AtomicI32::new(1);
    let notification_id = NOTIFICATION_ID.fetch_add(1, Ordering::Relaxed);

    if let Err(err) = app.notification().builder().id(notification_id).title(&title).body(&body).sound("default").show() {
        eprintln!("Failed to send notification: {}", err);
    }

    let mut count = state
        .unread_count
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *count += 1;
    update_tray_tooltip(&app, *count);
}

#[tauri::command]
pub fn update_badge_count(app: AppHandle, state: State<'_, AppState>, count: u32) {
    let mut unread = state
        .unread_count
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *unread = count;
    update_tray_tooltip(&app, count);
}

#[tauri::command]
pub fn report_user_active(state: State<'_, AppState>, conversation_id: Option<String>) {
    let mut active_id = state
        .active_conversation_id
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *active_id = conversation_id;
}

#[tauri::command]
pub fn get_app_config(app: AppHandle) -> AppConfig {
    load_app_config(&app)
}

#[tauri::command]
pub fn set_app_config(app: AppHandle, config: AppConfig) -> AppConfig {
    save_app_config(&app, &config);
    config
}

pub fn load_app_config(app: &AppHandle) -> AppConfig {
    let store = match app.store("store.json") {
        Ok(store) => store,
        Err(err) => {
            eprintln!("Failed to access store: {}", err);
            return AppConfig::default();
        }
    };

    match store.get("app_config") {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => {
            let config = AppConfig::default();
            if let Ok(value) = serde_json::to_value(&config) {
                store.set("app_config", value);
            }
            config
        }
    }
}

pub fn save_app_config(app: &AppHandle, config: &AppConfig) {
    let store = match app.store("store.json") {
        Ok(store) => store,
        Err(err) => {
            eprintln!("Failed to access store for save: {}", err);
            return;
        }
    };
    if let Ok(value) = serde_json::to_value(config) {
        store.set("app_config", value);
    }
}

pub fn load_window_state(app: &AppHandle) -> WindowState {
    let store = match app.store("store.json") {
        Ok(store) => store,
        Err(err) => {
            eprintln!("Failed to access store: {}", err);
            return WindowState::default();
        }
    };

    match store.get("window_state") {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => WindowState::default(),
    }
}

pub fn save_window_state(app: &AppHandle, window_state: &WindowState) {
    let store = match app.store("store.json") {
        Ok(store) => store,
        Err(err) => {
            eprintln!("Failed to access store for save: {}", err);
            return;
        }
    };
    if let Ok(value) = serde_json::to_value(window_state) {
        store.set("window_state", value);
    }
}

fn update_tray_tooltip(app: &AppHandle, count: u32) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let tooltip = if count > 0 {
            format!("Agent Playground — {} unread", count)
        } else {
            "Agent Playground".to_string()
        };
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}
