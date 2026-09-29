//! Global hotkeys, rebindable from settings. The plugin has one handler; this module maps
//! each registered shortcut id back to the action it triggers.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use crate::settings::Shortcuts;
use crate::{lyrics, overlay, settings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    ToggleEdit,
    ToggleVisible,
    OpenSettings,
    OffsetEarlier,
    OffsetLater,
    OffsetReset,
}

impl Action {
    fn label(self) -> &'static str {
        match self {
            Action::ToggleEdit => "Move / resize overlay",
            Action::ToggleVisible => "Show / hide overlay",
            Action::OpenSettings => "Open settings",
            Action::OffsetEarlier => "Show lyrics earlier",
            Action::OffsetLater => "Show lyrics later",
            Action::OffsetReset => "Reset lyrics offset",
        }
    }
}

fn bindings(s: &Shortcuts) -> [(Action, &str); 6] {
    [
        (Action::ToggleEdit, s.toggle_edit.as_str()),
        (Action::ToggleVisible, s.toggle_visible.as_str()),
        (Action::OpenSettings, s.open_settings.as_str()),
        (Action::OffsetEarlier, s.offset_earlier.as_str()),
        (Action::OffsetLater, s.offset_later.as_str()),
        (Action::OffsetReset, s.offset_reset.as_str()),
    ]
}

#[derive(Default)]
pub struct HotkeyState {
    by_id: Mutex<HashMap<u32, Action>>,
}

/// Every non-empty shortcut must parse and no two actions may share one.
/// An empty string leaves the action unbound.
pub fn validate(s: &Shortcuts) -> Result<(), String> {
    let mut seen: HashMap<u32, Action> = HashMap::new();
    for (action, accel) in bindings(s) {
        if accel.trim().is_empty() {
            continue;
        }
        let shortcut = Shortcut::from_str(accel)
            .map_err(|e| format!("\"{accel}\" for \"{}\" is not a valid shortcut: {e}", action.label()))?;
        if let Some(other) = seen.insert(shortcut.id(), action) {
            return Err(format!(
                "\"{accel}\" is assigned to both \"{}\" and \"{}\"",
                other.label(),
                action.label()
            ));
        }
    }
    Ok(())
}

/// Re-register all shortcuts. Returns a warning per shortcut the OS refused
/// (usually because another app already owns it).
pub fn apply<R: Runtime>(app: &AppHandle<R>, s: &Shortcuts) -> Vec<String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let state = app.state::<HotkeyState>();
    let mut by_id = state.by_id.lock().unwrap();
    by_id.clear();

    let mut warnings = Vec::new();
    for (action, accel) in bindings(s) {
        if accel.trim().is_empty() {
            continue;
        }
        let Ok(shortcut) = Shortcut::from_str(accel) else {
            warnings.push(format!("\"{accel}\" is not a valid shortcut"));
            continue;
        };
        match gs.register(shortcut) {
            Ok(()) => {
                by_id.insert(shortcut.id(), action);
            }
            Err(_) => warnings.push(format!(
                "{accel} (\"{}\") could not be registered. Another app may already be using it.",
                action.label()
            )),
        }
    }
    warnings
}

/// While the settings window records a new shortcut, the OS would swallow any combination we
/// already registered before the page sees it, so step aside until recording ends.
#[tauri::command]
pub fn suspend_hotkeys<R: Runtime>(app: AppHandle<R>) {
    let _ = app.global_shortcut().unregister_all();
    app.state::<HotkeyState>().by_id.lock().unwrap().clear();
}

#[tauri::command]
pub fn resume_hotkeys<R: Runtime>(app: AppHandle<R>) -> Vec<String> {
    apply(&app, &settings::current(&app).shortcuts)
}

pub fn handle<R: Runtime>(app: &AppHandle<R>, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state != ShortcutState::Pressed {
        return;
    }
    let action = app.state::<HotkeyState>().by_id.lock().unwrap().get(&shortcut.id()).copied();
    let Some(action) = action else {
        return;
    };
    let step = settings::current(app).offset_step_ms;
    match action {
        Action::ToggleEdit => overlay::toggle_edit_mode(app),
        Action::ToggleVisible => overlay::toggle_visible(app),
        Action::OpenSettings => crate::show_settings(app),
        Action::OffsetEarlier => lyrics::adjust_offset(app, Some(step)),
        Action::OffsetLater => lyrics::adjust_offset(app, Some(-step)),
        Action::OffsetReset => lyrics::adjust_offset(app, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        assert_eq!(validate(&Shortcuts::default()), Ok(()));
    }

    #[test]
    fn accepts_names_produced_by_the_settings_ui() {
        for accel in ["Ctrl+Alt+Equal", "Ctrl+Alt+Minus", "Ctrl+Shift+BracketLeft", "Alt+F9", "Ctrl+Alt+0"] {
            assert!(Shortcut::from_str(accel).is_ok(), "{accel}");
        }
    }

    #[test]
    fn rejects_garbage() {
        let s = Shortcuts {
            toggle_edit: "Ctrl+Alt+Banana".into(),
            ..Shortcuts::default()
        };
        assert!(validate(&s).unwrap_err().contains("Move / resize overlay"));
    }

    #[test]
    fn rejects_duplicates_even_with_different_spelling() {
        let s = Shortcuts {
            toggle_visible: "control+alt+l".into(),
            ..Shortcuts::default()
        };
        let err = validate(&s).unwrap_err();
        assert!(err.contains("both"), "{err}");
    }

    #[test]
    fn empty_means_unbound() {
        let s = Shortcuts {
            offset_reset: "".into(),
            ..Shortcuts::default()
        };
        assert_eq!(validate(&s), Ok(()));
    }
}
