//! Overlay window behaviour: click-through, never stealing focus, Lock/Edit mode.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Runtime, State, WebviewWindow};

pub const OVERLAY_LABEL: &str = "overlay";
pub const EVENT_EDIT_MODE: &str = "overlay://edit-mode";

/// `false` = Locked (click-through), `true` = Edit (drag & resize).
#[derive(Default)]
pub struct OverlayState {
    edit_mode: AtomicBool,
}

pub fn overlay_window<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window(OVERLAY_LABEL)
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
    app.state::<OverlayState>()
        .edit_mode
        .store(edit, Ordering::SeqCst);
    if let Some(window) = overlay_window(app) {
        let _ = window.set_ignore_cursor_events(!edit);
        apply_overlay_exstyle(&window);
    }
    let _ = app.emit(EVENT_EDIT_MODE, edit);
}

pub fn toggle_edit_mode<R: Runtime>(app: &AppHandle<R>) {
    let edit = app.state::<OverlayState>().edit_mode.load(Ordering::SeqCst);
    set_edit_mode(app, !edit);
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

pub fn init<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let window = overlay_window(app).expect("overlay window missing from tauri.conf.json");
    place_bottom_center(&window);
    set_edit_mode(app, false);
    window.show()?;
    apply_overlay_exstyle(&window);
    Ok(())
}

/// Lets the overlay page read the current mode on load, since it may miss the first event.
#[tauri::command]
pub fn get_edit_mode(state: State<'_, OverlayState>) -> bool {
    state.edit_mode.load(Ordering::SeqCst)
}
