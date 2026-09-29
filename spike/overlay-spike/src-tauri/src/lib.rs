// Tahap 0 spike: a transparent, always-on-top, click-through overlay window.
//
// Hotkeys:
//   Ctrl+Alt+L  toggle Lock (click-through) / Edit (drag & resize)
//   Ctrl+Alt+H  show / hide overlay
//   Ctrl+Alt+Q  quit

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Emitter, Manager, PhysicalPosition, WebviewWindow};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

static EDIT_MODE: AtomicBool = AtomicBool::new(false);

/// Keep the overlay out of Alt+Tab and stop it from ever taking focus away from the game.
/// Must be re-applied after `set_ignore_cursor_events`, which rewrites the extended style.
#[cfg(windows)]
fn apply_overlay_exstyle(window: &WebviewWindow) {
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
fn apply_overlay_exstyle(_window: &WebviewWindow) {}

fn set_edit_mode(window: &WebviewWindow, edit: bool) {
    EDIT_MODE.store(edit, Ordering::SeqCst);
    let _ = window.set_ignore_cursor_events(!edit);
    apply_overlay_exstyle(window);
    let _ = window.emit("edit-mode", edit);
}

/// Place the overlay at the bottom-center of the primary monitor.
fn place_bottom_center(window: &WebviewWindow) {
    let (Ok(Some(monitor)), Ok(size)) = (window.primary_monitor(), window.outer_size()) else {
        return;
    };
    let area = monitor.size();
    let origin = monitor.position();
    let x = origin.x + (area.width as i32 - size.width as i32) / 2;
    let y = origin.y + area.height as i32 - size.height as i32 - (area.height as i32 / 8);
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let toggle_edit = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyL);
    let toggle_visible = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyH);
    let quit = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyQ);

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts([toggle_edit, toggle_visible, quit])
                .expect("failed to register global shortcuts")
                .with_handler(move |app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    let Some(window) = app.get_webview_window("main") else {
                        return;
                    };
                    if shortcut == &toggle_edit {
                        set_edit_mode(&window, !EDIT_MODE.load(Ordering::SeqCst));
                    } else if shortcut == &toggle_visible {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            apply_overlay_exstyle(&window);
                        }
                    } else if shortcut == &quit {
                        app.exit(0);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .expect("main window missing");
            place_bottom_center(&window);
            set_edit_mode(&window, false);
            window.show()?;
            apply_overlay_exstyle(&window);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
