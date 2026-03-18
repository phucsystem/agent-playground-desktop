use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

use crate::commands::load_app_config;

pub fn check_for_updates_background(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;

        let config = load_app_config(&handle);
        if !config.check_updates {
            return;
        }

        run_update_check(&handle, false).await;
    });
}

pub async fn run_update_check(app: &AppHandle, manual: bool) {
    let updater = match app.updater() {
        Ok(updater) => updater,
        Err(err) => {
            eprintln!("[updater] Failed to get updater: {}", err);
            if manual {
                let _ = app
                    .notification()
                    .builder()
                    .title("Update Check Failed")
                    .body(format!("Could not check for updates: {}", err))
                    .show();
            }
            return;
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let _ = app
                .notification()
                .builder()
                .title("Update Available")
                .body(format!(
                    "Agent Playground v{} is available. Downloading...",
                    version
                ))
                .show();

            match update.download_and_install(|_, _| {}, || {}).await {
                Ok(_) => {
                    let _ = app
                        .notification()
                        .builder()
                        .title("Update Installed")
                        .body(format!(
                            "v{} installed. Restart the app to apply.",
                            version
                        ))
                        .show();
                }
                Err(err) => {
                    eprintln!("[updater] Download/install failed: {}", err);
                    let _ = app
                        .notification()
                        .builder()
                        .title("Update Failed")
                        .body(format!("Failed to install update: {}", err))
                        .show();
                }
            }
        }
        Ok(None) => {
            if manual {
                let _ = app
                    .notification()
                    .builder()
                    .title("No Updates")
                    .body("You're running the latest version.")
                    .show();
            }
        }
        Err(err) => {
            eprintln!("[updater] Check failed: {}", err);
            if manual {
                let _ = app
                    .notification()
                    .builder()
                    .title("Update Check Failed")
                    .body(format!("Could not check for updates: {}", err))
                    .show();
            }
        }
    }
}
