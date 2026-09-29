//! Self-update from GitHub Releases (endpoint and public key in tauri.conf.json).
//! Release builds check quietly shortly after start; Settings can check and install.

use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::overlay;

pub const EVENT_UPDATE: &str = "update://available";

/// Let the app settle (and the network come up after a Windows login) before checking.
const STARTUP_DELAY: Duration = Duration::from_secs(20);

#[derive(Default)]
pub struct UpdateState {
    pending: Mutex<Option<Update>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
}

impl From<&Update> for UpdateInfo {
    fn from(u: &Update) -> Self {
        Self {
            version: u.version.clone(),
            current_version: u.current_version.clone(),
            notes: u.body.clone().filter(|b| !b.trim().is_empty()),
        }
    }
}

async fn check<R: Runtime>(app: &AppHandle<R>) -> Result<Option<UpdateInfo>, String> {
    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;
    let info = update.as_ref().map(UpdateInfo::from);
    *app.state::<UpdateState>().pending.lock().unwrap() = update;
    Ok(info)
}

/// Quiet background check for release builds; announces a new version once.
pub fn spawn_startup_check<R: Runtime>(app: AppHandle<R>) {
    if cfg!(debug_assertions) {
        return; // Dev builds would only ever find the latest public release.
    }
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(STARTUP_DELAY).await;
        match check(&app).await {
            Ok(Some(info)) => {
                overlay::toast(&app, &format!("Konco Nembang {} is available. Open Settings to update.", info.version));
                let _ = app.emit(EVENT_UPDATE, &info);
            }
            Ok(None) => {}
            Err(err) => eprintln!("[update] check failed: {err}"),
        }
    });
}

#[tauri::command]
pub async fn check_for_update<R: Runtime>(app: AppHandle<R>) -> Result<Option<UpdateInfo>, String> {
    check(&app).await
}

/// An update found earlier (e.g. by the startup check), for a Settings window opened later.
#[tauri::command]
pub fn pending_update(state: State<'_, UpdateState>) -> Option<UpdateInfo> {
    state.pending.lock().unwrap().as_ref().map(UpdateInfo::from)
}

/// Download, verify the signature and run the installer; the app restarts into the new version.
#[tauri::command]
pub async fn install_update<R: Runtime>(app: AppHandle<R>, state: State<'_, UpdateState>) -> Result<(), String> {
    let update = state.pending.lock().unwrap().clone();
    let Some(update) = update else {
        return Err("No update is ready. Check for updates first.".into());
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;
    // On Windows the installer usually closes the app itself; this covers the rest.
    app.restart();
}
