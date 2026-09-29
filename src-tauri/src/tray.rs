//! System tray icon: the app's home now that it has no main window.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Runtime};

use crate::overlay;

const TRAY_ID: &str = "main";
const APP_NAME: &str = "Konco Nembang";
/// Windows truncates tray tooltips at 127 characters.
const MAX_TOOLTIP_CHARS: usize = 120;

pub fn init<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let toggle_visible = MenuItem::with_id(app, "toggle-visible", "Show / hide overlay", true, None::<&str>)?;
    let toggle_edit = MenuItem::with_id(app, "toggle-edit", "Move / resize overlay", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Konco Nembang", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &toggle_visible,
            &toggle_edit,
            &PredefinedMenuItem::separator(app)?,
            &settings,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(APP_NAME)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle-visible" => overlay::toggle_visible(app),
            "toggle-edit" => overlay::toggle_edit_mode(app),
            "settings" => crate::show_settings(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::show_settings(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

pub fn tooltip_for(title: Option<&str>, artist: Option<&str>) -> String {
    let track = match (title, artist) {
        (Some(t), Some(a)) if !t.is_empty() && !a.is_empty() => format!("{t} - {a}"),
        (Some(t), _) if !t.is_empty() => t.to_string(),
        _ => return APP_NAME.to_string(),
    };
    let text = format!("{APP_NAME}\n{track}");
    if text.chars().count() <= MAX_TOOLTIP_CHARS {
        text
    } else {
        let cut: String = text.chars().take(MAX_TOOLTIP_CHARS - 1).collect();
        format!("{cut}…")
    }
}

pub fn set_tooltip<R: Runtime>(app: &AppHandle<R>, text: &str) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(text));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_shows_track() {
        assert_eq!(
            tooltip_for(Some("So Far Away"), Some("Avenged Sevenfold")),
            "Konco Nembang\nSo Far Away - Avenged Sevenfold"
        );
        assert_eq!(tooltip_for(Some("Intro"), Some("")), "Konco Nembang\nIntro");
        assert_eq!(tooltip_for(None, None), "Konco Nembang");
    }

    #[test]
    fn tooltip_is_truncated() {
        let long = "x".repeat(300);
        let tip = tooltip_for(Some(&long), Some("A"));
        assert_eq!(tip.chars().count(), MAX_TOOLTIP_CHARS);
        assert!(tip.ends_with('…'));
    }
}
