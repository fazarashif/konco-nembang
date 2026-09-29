//! System tray icon: the app's home now that it has no main window. The monochrome icon
//! follows the taskbar theme (white on dark, black on light) and shows the overlay state.

use std::sync::atomic::{AtomicU8, Ordering};
use std::thread;
use std::time::Duration;

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Runtime};

use crate::overlay;

const TRAY_ID: &str = "main";
const APP_NAME: &str = "Konco Nembang";
/// Windows truncates tray tooltips at 127 characters.
const MAX_TOOLTIP_CHARS: usize = 120;
/// Windows has no cheap "taskbar theme changed" hook for us, so check now and then.
const THEME_POLL: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayState {
    Normal,
    Hidden,
    Edit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    /// For dark taskbars.
    White,
    /// For light taskbars.
    Black,
}

macro_rules! tray_png {
    ($state:literal, $color:literal, $size:literal) => {
        include_bytes!(concat!("../icons/tray/tray-", $state, "-", $color, "-", $size, ".png"))
    };
}

macro_rules! by_size {
    ($state:literal, $color:literal, $px:expr) => {
        match $px {
            16 => tray_png!($state, $color, "16").as_slice(),
            24 => tray_png!($state, $color, "24").as_slice(),
            _ => tray_png!($state, $color, "32").as_slice(),
        }
    };
}

fn icon_bytes(state: TrayState, glyph: Glyph, px: u32) -> &'static [u8] {
    match (state, glyph) {
        (TrayState::Normal, Glyph::White) => by_size!("normal", "white", px),
        (TrayState::Normal, Glyph::Black) => by_size!("normal", "black", px),
        (TrayState::Hidden, Glyph::White) => by_size!("hidden", "white", px),
        (TrayState::Hidden, Glyph::Black) => by_size!("hidden", "black", px),
        (TrayState::Edit, Glyph::White) => by_size!("edit", "white", px),
        (TrayState::Edit, Glyph::Black) => by_size!("edit", "black", px),
    }
}

/// Tray icons are 16 px at 100% scaling; pick the drawn size closest to what Windows shows.
pub fn icon_px(scale_factor: f64) -> u32 {
    if scale_factor >= 1.75 {
        32
    } else if scale_factor >= 1.25 {
        24
    } else {
        16
    }
}

#[cfg(windows)]
fn taskbar_is_light() -> bool {
    use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
    let key: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let value: Vec<u16> = "SystemUsesLightTheme\0".encode_utf16().collect();
    let mut data: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&mut data as *mut u32).cast(),
            &mut size,
        )
    };
    // Missing value (older Windows): the taskbar is dark.
    status == 0 && data == 1
}

#[cfg(not(windows))]
fn taskbar_is_light() -> bool {
    false
}

fn current_glyph() -> Glyph {
    if taskbar_is_light() {
        Glyph::Black
    } else {
        Glyph::White
    }
}

/// Last icon set, packed as state | glyph << 2 | size bucket << 3, to skip redundant updates.
static LAST_ICON: AtomicU8 = AtomicU8::new(u8::MAX);

fn pack(state: TrayState, glyph: Glyph, px: u32) -> u8 {
    let s = match state {
        TrayState::Normal => 0,
        TrayState::Hidden => 1,
        TrayState::Edit => 2,
    };
    let g = if glyph == Glyph::Black { 1 } else { 0 };
    let p = match px {
        16 => 0,
        24 => 1,
        _ => 2,
    };
    s | (g << 2) | (p << 3)
}

fn current_state<R: Runtime>(app: &AppHandle<R>) -> TrayState {
    if overlay::is_editing(app) {
        TrayState::Edit
    } else if !overlay::is_visible(app) {
        TrayState::Hidden
    } else {
        TrayState::Normal
    }
}

fn scale_factor<R: Runtime>(app: &AppHandle<R>) -> f64 {
    app.primary_monitor()
        .ok()
        .flatten()
        .map(|m| m.scale_factor())
        .unwrap_or(1.0)
}

/// Update the icon to match the overlay state and taskbar theme. Cheap when nothing changed.
pub fn refresh<R: Runtime>(app: &AppHandle<R>) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let (state, glyph, px) = (current_state(app), current_glyph(), icon_px(scale_factor(app)));
    let packed = pack(state, glyph, px);
    if LAST_ICON.swap(packed, Ordering::SeqCst) == packed {
        return;
    }
    match Image::from_bytes(icon_bytes(state, glyph, px)) {
        Ok(icon) => {
            let _ = tray.set_icon(Some(icon));
        }
        Err(err) => eprintln!("[tray] bad icon: {err}"),
    }
}

pub fn init<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let toggle_visible = MenuItem::with_id(app, "toggle-visible", "Show / hide overlay", true, None::<&str>)?;
    let toggle_edit = MenuItem::with_id(app, "toggle-edit", "Move / resize overlay", true, None::<&str>)?;
    let search = MenuItem::with_id(app, "search", "Search lyrics…", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Konco Nembang", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &toggle_visible,
            &toggle_edit,
            &PredefinedMenuItem::separator(app)?,
            &search,
            &settings,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let px = icon_px(scale_factor(app));
    let icon = Image::from_bytes(icon_bytes(TrayState::Normal, current_glyph(), px))?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip(APP_NAME)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle-visible" => overlay::toggle_visible(app),
            "toggle-edit" => overlay::toggle_edit_mode(app),
            "search" => crate::show_search(app),
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
        })
        .build(app)?;
    refresh(app);

    let app = app.clone();
    thread::Builder::new()
        .name("tray-theme".into())
        .spawn(move || loop {
            thread::sleep(THEME_POLL);
            refresh(&app);
        })
        .expect("failed to spawn tray theme thread");
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

    #[test]
    fn icon_size_follows_dpi() {
        assert_eq!(icon_px(1.0), 16);
        assert_eq!(icon_px(1.25), 24);
        assert_eq!(icon_px(1.5), 24);
        assert_eq!(icon_px(2.0), 32);
    }

    #[test]
    fn every_icon_decodes() {
        for state in [TrayState::Normal, TrayState::Hidden, TrayState::Edit] {
            for glyph in [Glyph::White, Glyph::Black] {
                for px in [16, 24, 32] {
                    let img = Image::from_bytes(icon_bytes(state, glyph, px)).unwrap();
                    assert_eq!((img.width(), img.height()), (px, px), "{state:?} {glyph:?} {px}");
                }
            }
        }
    }

    #[test]
    fn packed_keys_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for state in [TrayState::Normal, TrayState::Hidden, TrayState::Edit] {
            for glyph in [Glyph::White, Glyph::Black] {
                for px in [16, 24, 32] {
                    assert!(seen.insert(pack(state, glyph, px)));
                }
            }
        }
    }
}
