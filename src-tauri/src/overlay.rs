//! Overlay window behaviour: click-through, never stealing focus, Lock/Edit mode, and
//! remembering where the user placed it.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Runtime, State,
    WebviewWindow,
};

pub const OVERLAY_LABEL: &str = "overlay";
pub const EVENT_EDIT_MODE: &str = "overlay://edit-mode";
pub const EVENT_TOAST: &str = "overlay://toast";

/// Matches the overlay size in tauri.conf.json.
const DEFAULT_SIZE: LogicalSize<f64> = LogicalSize::new(900.0, 170.0);

/// How much of a saved overlay must still land on a monitor to reuse the saved spot.
const MIN_VISIBLE_WIDTH: i64 = 120;
const MIN_VISIBLE_HEIGHT: i64 = 40;

/// `false` = Locked (click-through), `true` = Edit (drag & resize).
#[derive(Default)]
pub struct OverlayState {
    edit_mode: AtomicBool,
}

/// Overlay placement in physical pixels, persisted to `<app data>/overlay.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Monitor work area in physical pixels.
#[derive(Clone, Copy, Debug)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// True if enough of `g` overlaps some monitor for the user to see and grab it.
/// Guards against a saved spot on a monitor that has since been unplugged.
pub fn is_reachable(g: &Geometry, monitors: &[Area]) -> bool {
    monitors.iter().any(|m| {
        let left = (g.x as i64).max(m.x as i64);
        let right = (g.x as i64 + g.width as i64).min(m.x as i64 + m.width as i64);
        let top = (g.y as i64).max(m.y as i64);
        let bottom = (g.y as i64 + g.height as i64).min(m.y as i64 + m.height as i64);
        right - left >= MIN_VISIBLE_WIDTH && bottom - top >= MIN_VISIBLE_HEIGHT
    })
}

pub fn overlay_window<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window(OVERLAY_LABEL)
}

pub fn is_editing<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.state::<OverlayState>().edit_mode.load(Ordering::SeqCst)
}

pub fn is_visible<R: Runtime>(app: &AppHandle<R>) -> bool {
    overlay_window(app).is_some_and(|w| w.is_visible().unwrap_or(false))
}

fn geometry_path<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path()
        .app_data_dir()
        .ok()
        .map(|dir| dir.join("overlay.json"))
}

fn load_geometry<R: Runtime>(app: &AppHandle<R>) -> Option<Geometry> {
    let data = fs::read_to_string(geometry_path(app)?).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_geometry<R: Runtime>(app: &AppHandle<R>) {
    let (Some(window), Some(path)) = (overlay_window(app), geometry_path(app)) else {
        return;
    };
    let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) else {
        return;
    };
    let geometry = Geometry {
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
    };
    let result = serde_json::to_string(&geometry)
        .map_err(|e| e.to_string())
        .and_then(|json| {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            fs::write(&path, json).map_err(|e| e.to_string())
        });
    if let Err(err) = result {
        eprintln!("[overlay] could not save position: {err}");
    }
}

/// Keep the overlay out of Alt+Tab and stop it from ever taking focus away from the game.
/// Must be re-applied after `set_ignore_cursor_events`, which rewrites the extended style.
#[cfg(windows)]
fn apply_overlay_exstyle<R: Runtime>(window: &WebviewWindow<R>) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    };
    if let Ok(hwnd) = window.hwnd() {
        let hwnd = hwnd.0 as _;
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                style | (WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW) as isize,
            );
        }
    }
}

#[cfg(not(windows))]
fn apply_overlay_exstyle<R: Runtime>(_window: &WebviewWindow<R>) {}

pub fn set_edit_mode<R: Runtime>(app: &AppHandle<R>, edit: bool) {
    let was_editing = app
        .state::<OverlayState>()
        .edit_mode
        .swap(edit, Ordering::SeqCst);
    if let Some(window) = overlay_window(app) {
        let _ = window.set_ignore_cursor_events(!edit);
        apply_overlay_exstyle(&window);
    }
    // Locking again is the natural "done moving it" moment.
    if was_editing && !edit {
        save_geometry(app);
    }
    let _ = app.emit(EVENT_EDIT_MODE, edit);
    crate::tray::refresh(app);
}

/// On quit: keep a placement the user was still adjusting. A placement that was never
/// touched is not saved, so the default keeps following the primary monitor.
pub fn save_if_editing<R: Runtime>(app: &AppHandle<R>) {
    if is_editing(app) {
        save_geometry(app);
    }
}

pub fn toggle_edit_mode<R: Runtime>(app: &AppHandle<R>) {
    set_edit_mode(app, !is_editing(app));
}

pub fn toggle_visible<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = overlay_window(app) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        apply_overlay_exstyle(&window);
    }
    crate::tray::refresh(app);
}

/// Place the overlay at the bottom-center of the primary monitor.
fn place_bottom_center<R: Runtime>(window: &WebviewWindow<R>) {
    let (Ok(Some(monitor)), Ok(size)) = (window.primary_monitor(), window.outer_size()) else {
        return;
    };
    let area = monitor.size();
    let origin = monitor.position();
    let x = origin.x + (area.width as i32 - size.width as i32) / 2;
    let y = origin.y + area.height as i32 - size.height as i32 - (area.height as i32 / 8);
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

/// Restore the saved spot if it is still on a connected monitor, else use the default.
fn restore_or_place<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    let monitors: Vec<Area> = window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| Area {
            x: m.position().x,
            y: m.position().y,
            width: m.size().width,
            height: m.size().height,
        })
        .collect();

    match load_geometry(app) {
        Some(g) if is_reachable(&g, &monitors) => {
            let _ = window.set_size(PhysicalSize::new(g.width, g.height));
            let _ = window.set_position(PhysicalPosition::new(g.x, g.y));
        }
        _ => place_bottom_center(window),
    }
}

pub fn init<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let window = overlay_window(app).expect("overlay window missing from tauri.conf.json");
    restore_or_place(app, &window);
    set_edit_mode(app, false);
    window.show()?;
    apply_overlay_exstyle(&window);
    Ok(())
}

/// Short feedback message shown on the overlay (e.g. after an offset hotkey).
pub fn toast<R: Runtime>(app: &AppHandle<R>, message: &str) {
    let _ = app.emit_to(OVERLAY_LABEL, EVENT_TOAST, message);
}

/// Lets the overlay page read the current mode on load, since it may miss the first event.
#[tauri::command]
pub fn get_edit_mode(state: State<'_, OverlayState>) -> bool {
    state.edit_mode.load(Ordering::SeqCst)
}

/// Forget the saved placement and put the overlay back at the default size and spot.
#[tauri::command]
pub fn reset_overlay_position<R: Runtime>(app: AppHandle<R>) {
    if let Some(path) = geometry_path(&app) {
        let _ = fs::remove_file(path);
    }
    if let Some(window) = overlay_window(&app) {
        let _ = window.set_size(DEFAULT_SIZE);
        place_bottom_center(&window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIMARY: Area = Area { x: 0, y: 0, width: 2560, height: 1440 };
    const LEFT: Area = Area { x: -1920, y: 360, width: 1920, height: 1080 };

    fn g(x: i32, y: i32) -> Geometry {
        Geometry { x, y, width: 900, height: 170 }
    }

    #[test]
    fn fully_on_screen() {
        assert!(is_reachable(&g(830, 1090), &[PRIMARY]));
    }

    #[test]
    fn on_secondary_monitor_with_negative_coordinates() {
        assert!(is_reachable(&g(-1500, 900), &[PRIMARY, LEFT]));
    }

    #[test]
    fn secondary_monitor_unplugged() {
        assert!(!is_reachable(&g(-1500, 900), &[PRIMARY]));
    }

    #[test]
    fn mostly_off_screen_but_grabbable() {
        // 200 px still visible on the right edge.
        assert!(is_reachable(&g(2360, 600), &[PRIMARY]));
    }

    #[test]
    fn only_a_sliver_visible() {
        assert!(!is_reachable(&g(2500, 600), &[PRIMARY]));
        assert!(!is_reachable(&g(830, 1420), &[PRIMARY]));
    }

    #[test]
    fn no_monitors_reported() {
        assert!(!is_reachable(&g(0, 0), &[]));
    }
}
