mod hotkeys;
mod lyrics;
mod media;
mod overlay;
mod settings;
mod tray;

use tauri::{AppHandle, Manager, RunEvent, Runtime, WindowEvent};

const SETTINGS_LABEL: &str = "settings";

pub(crate) fn show_settings<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be registered first: a second launch just opens settings in the running app.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_settings(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(hotkeys::handle)
                .build(),
        )
        .manage(overlay::OverlayState::default())
        .manage(media::MediaState::default())
        .manage(hotkeys::HotkeyState::default())
        .invoke_handler(tauri::generate_handler![
            overlay::get_edit_mode,
            overlay::reset_overlay_position,
            media::get_now_playing,
            lyrics::get_lyrics,
            lyrics::adjust_track_offset,
            settings::get_settings,
            settings::update_settings,
            hotkeys::suspend_hotkeys,
            hotkeys::resume_hotkeys
        ])
        .setup(|app| {
            let handle = app.handle();
            app.manage(settings::SettingsState::load(handle));
            for warning in settings::apply(handle, &settings::current(handle)) {
                eprintln!("[settings] {warning}");
            }
            overlay::init(handle)?;
            app.manage(lyrics::LyricsState::new(handle));
            tray::init(handle)?;
            media::spawn_watcher(handle.clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing settings only hides it; the app keeps running in the tray.
            if window.label() == SETTINGS_LABEL {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // Windows still exist at this point, unlike at `RunEvent::Exit`.
            if let RunEvent::ExitRequested { .. } = event {
                overlay::save_if_editing(app);
            }
        });
}
