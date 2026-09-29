mod hotkeys;
mod lyrics;
mod media;
mod overlay;
mod settings;
mod tray;
mod updates;

use tauri::{AppHandle, Manager, RunEvent, Runtime, WebviewUrl, WebviewWindowBuilder};

const SETTINGS_LABEL: &str = "settings";
const SEARCH_LABEL: &str = "search";

/// Tool windows are created on demand and destroyed on close: each open webview costs
/// tens of MB, and most of the time only the overlay is needed.
fn show_window<R: Runtime>(
    app: &AppHandle<R>,
    label: &str,
    build: impl FnOnce(WebviewWindowBuilder<'_, R, AppHandle<R>>) -> WebviewWindowBuilder<'_, R, AppHandle<R>>,
    page: &str,
) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let builder = WebviewWindowBuilder::new(app, label, WebviewUrl::App(page.into()))
        .resizable(true)
        .center();
    if let Err(err) = build(builder).build() {
        eprintln!("[window] could not open {label}: {err}");
    }
}

pub(crate) fn show_settings<R: Runtime>(app: &AppHandle<R>) {
    show_window(
        app,
        SETTINGS_LABEL,
        |b| b.title("Konco Nembang Settings").inner_size(520.0, 640.0).min_inner_size(440.0, 400.0),
        "settings.html",
    );
}

pub(crate) fn show_search<R: Runtime>(app: &AppHandle<R>) {
    show_window(
        app,
        SEARCH_LABEL,
        |b| {
            b.title("Konco Nembang Search Lyrics")
                .inner_size(760.0, 560.0)
                .min_inner_size(560.0, 400.0)
                // Let the page receive .lrc files dropped on it as regular DOM drop events.
                .disable_drag_drop_handler()
        },
        "search.html",
    );
}

#[tauri::command]
fn open_search<R: Runtime>(app: AppHandle<R>) {
    show_search(&app);
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(updates::UpdateState::default())
        .manage(overlay::OverlayState::default())
        .manage(media::MediaState::default())
        .manage(hotkeys::HotkeyState::default())
        .invoke_handler(tauri::generate_handler![
            overlay::get_edit_mode,
            overlay::reset_overlay_position,
            media::get_now_playing,
            lyrics::get_lyrics,
            lyrics::adjust_track_offset,
            lyrics::get_search_defaults,
            lyrics::search_lyrics,
            lyrics::apply_manual_lyrics,
            lyrics::refresh_lyrics,
            open_search,
            settings::get_settings,
            settings::update_settings,
            hotkeys::suspend_hotkeys,
            hotkeys::resume_hotkeys,
            updates::check_for_update,
            updates::pending_update,
            updates::install_update
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
            updates::spawn_startup_check(handle.clone());
            Ok(())
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
