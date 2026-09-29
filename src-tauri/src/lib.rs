mod media;
mod overlay;

use tauri::{AppHandle, Manager, Runtime, WindowEvent};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

const SETTINGS_LABEL: &str = "settings";

fn show_settings<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Temporary hotkeys until the tray menu lands (Tahap 5).
    let ctrl_alt = Some(Modifiers::CONTROL | Modifiers::ALT);
    let toggle_edit = Shortcut::new(ctrl_alt, Code::KeyL);
    let toggle_visible = Shortcut::new(ctrl_alt, Code::KeyH);
    let open_settings = Shortcut::new(ctrl_alt, Code::KeyS);
    let quit = Shortcut::new(ctrl_alt, Code::KeyQ);

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts([toggle_edit, toggle_visible, open_settings, quit])
                .expect("failed to register global shortcuts")
                .with_handler(move |app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    if shortcut == &toggle_edit {
                        overlay::toggle_edit_mode(app);
                    } else if shortcut == &toggle_visible {
                        overlay::toggle_visible(app);
                    } else if shortcut == &open_settings {
                        show_settings(app);
                    } else if shortcut == &quit {
                        app.exit(0);
                    }
                })
                .build(),
        )
        .manage(overlay::OverlayState::default())
        .manage(media::MediaState::default())
        .invoke_handler(tauri::generate_handler![
            overlay::get_edit_mode,
            media::get_now_playing
        ])
        .setup(|app| {
            overlay::init(app.handle())?;
            media::spawn_watcher(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing settings only hides it; the app keeps running for the overlay.
            if window.label() == SETTINGS_LABEL {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
