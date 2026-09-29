//! User settings, owned by the backend and persisted to `<app data>/settings.json`.
//! Windows read them with `get_settings` and follow `settings://changed`.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::{hotkeys, media};

pub const EVENT_SETTINGS: &str = "settings://changed";

pub const MAX_OFFSET_MS: i32 = 30_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Left,
    Center,
    Right,
}

/// Overlay style presets from the design system (`data-kn-preset` on the overlay frame).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    Minimal,
    Pill,
    Karaoke,
    Subtle,
}

/// Which players the overlay follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaSourceFilter {
    Auto,
    Spotify,
    Browser,
}

/// Accelerator strings such as `Ctrl+Alt+L`, parsed by the global-hotkey crate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Shortcuts {
    pub toggle_edit: String,
    pub toggle_visible: String,
    pub open_settings: String,
    /// Lyrics come too late: show them earlier.
    pub offset_earlier: String,
    /// Lyrics come too early: show them later.
    pub offset_later: String,
    pub offset_reset: String,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            toggle_edit: "Ctrl+Alt+L".into(),
            toggle_visible: "Ctrl+Alt+H".into(),
            open_settings: "Ctrl+Alt+S".into(),
            offset_earlier: "Ctrl+Alt+Equal".into(),
            offset_later: "Ctrl+Alt+Minus".into(),
            offset_reset: "Ctrl+Alt+0".into(),
        }
    }
}

/// Current settings file format. Bump together with a step in `Settings::migrated`.
const SETTINGS_VERSION: u32 = 2;

/// Files written before the version field existed.
fn legacy_version() -> u32 {
    1
}

/// Missing fields fall back to defaults, so older settings files keep loading.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "legacy_version")]
    pub version: u32,
    // Appearance
    pub preset: Preset,
    pub visible_lines: u8,
    /// Multiplier on top of the size the overlay window gives the text.
    pub text_scale: f32,
    pub align: Align,
    /// The fields below override the preset; `None` keeps the preset's own value.
    pub text_opacity: Option<f32>,
    /// Opacity of the dark backdrop behind the lyrics; 0 = no background.
    pub background_opacity: Option<f32>,
    /// CSS hex color of the active line.
    pub active_color: Option<String>,
    pub animations: bool,
    // Behavior
    pub hide_paused_after_ms: Option<u32>,
    pub hide_no_lyrics_after_ms: Option<u32>,
    pub media_source: MediaSourceFilter,
    // Sync
    /// Added to every track; positive shows lyrics earlier.
    pub global_offset_ms: i32,
    pub offset_step_ms: i32,
    // Shortcuts
    pub shortcuts: Shortcuts,
    // General
    pub autostart: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            preset: Preset::Minimal,
            visible_lines: 3,
            text_scale: 1.0,
            align: Align::Center,
            text_opacity: None,
            background_opacity: None,
            active_color: None,
            animations: true,
            // The overlay dims right away when paused and hides after this.
            hide_paused_after_ms: Some(10_000),
            hide_no_lyrics_after_ms: Some(6000),
            media_source: MediaSourceFilter::Auto,
            global_offset_ms: 0,
            offset_step_ms: 250,
            shortcuts: Shortcuts::default(),
            autostart: false,
        }
    }
}

fn is_hex_color(s: &str) -> bool {
    let hex = s.strip_prefix('#').unwrap_or("");
    matches!(hex.len(), 3 | 6) && hex.chars().all(|c| c.is_ascii_hexdigit())
}

fn clamp_opt(v: Option<f32>, min: f32, max: f32) -> Option<f32> {
    v.filter(|x| x.is_finite()).map(|x| x.clamp(min, max))
}

impl Settings {
    /// Bring files from older versions up to date.
    pub fn migrated(mut self) -> Self {
        if self.version < 2 {
            // v1 stored its defaults explicitly, which would now pin every preset to plain
            // white text without a backdrop. Only values the user actually changed survive.
            if self.active_color.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("#ffffff")) {
                self.active_color = None;
            }
            if self.text_opacity == Some(1.0) {
                self.text_opacity = None;
            }
            if self.background_opacity == Some(0.0) {
                self.background_opacity = None;
            }
            if self.hide_paused_after_ms == Some(3000) {
                self.hide_paused_after_ms = Some(10_000);
            }
        }
        self.version = SETTINGS_VERSION;
        self
    }

    /// Clamp everything into range so a hand-edited or buggy value cannot break the overlay.
    pub fn sanitized(mut self) -> Self {
        self.visible_lines = self.visible_lines.clamp(1, 3);
        self.text_scale = if self.text_scale.is_finite() { self.text_scale.clamp(0.5, 2.5) } else { 1.0 };
        self.text_opacity = clamp_opt(self.text_opacity, 0.2, 1.0);
        self.background_opacity = clamp_opt(self.background_opacity, 0.0, 0.9);
        if !self.active_color.as_deref().is_none_or(is_hex_color) {
            self.active_color = None;
        }
        self.hide_paused_after_ms = self.hide_paused_after_ms.map(|ms| ms.min(600_000));
        self.hide_no_lyrics_after_ms = self.hide_no_lyrics_after_ms.map(|ms| ms.min(600_000));
        self.global_offset_ms = self.global_offset_ms.clamp(-MAX_OFFSET_MS, MAX_OFFSET_MS);
        self.offset_step_ms = self.offset_step_ms.clamp(50, 2000);
        self
    }
}

pub struct SettingsState {
    current: Mutex<Settings>,
    path: Option<PathBuf>,
}

impl SettingsState {
    pub fn load<R: Runtime>(app: &AppHandle<R>) -> Self {
        let path = app.path().app_data_dir().ok().map(|d| d.join("settings.json"));
        let current = path
            .as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|data| serde_json::from_str::<Settings>(&data).ok())
            .unwrap_or_default()
            .migrated()
            .sanitized();
        Self {
            current: Mutex::new(current),
            path,
        }
    }

    pub fn get(&self) -> Settings {
        self.current.lock().unwrap().clone()
    }

    fn save(&self, settings: &Settings) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Err("no app data directory".into());
        };
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json)
            .and_then(|_| fs::rename(&tmp, path))
            .map_err(|e| e.to_string())
    }
}

pub fn current<R: Runtime>(app: &AppHandle<R>) -> Settings {
    app.state::<SettingsState>().get()
}

#[cfg(desktop)]
fn apply_autostart<R: Runtime>(app: &AppHandle<R>, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let launcher = app.autolaunch();
    if launcher.is_enabled().unwrap_or(false) == enabled {
        return Ok(());
    }
    let result = if enabled { launcher.enable() } else { launcher.disable() };
    result.map_err(|e| e.to_string())
}

/// Push settings into the parts of the backend that depend on them.
/// Returns human-readable warnings (e.g. a shortcut another app already owns).
pub fn apply<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> Vec<String> {
    let mut warnings = hotkeys::apply(app, &settings.shortcuts);
    media::set_source_filter(app, settings.media_source);
    #[cfg(desktop)]
    if let Err(err) = apply_autostart(app, settings.autostart) {
        warnings.push(format!("Could not change start with Windows: {err}"));
    }
    warnings
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResult {
    settings: Settings,
    warnings: Vec<String>,
}

#[tauri::command]
pub fn get_settings(state: State<'_, SettingsState>) -> Settings {
    state.get()
}

#[tauri::command]
pub fn update_settings<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, SettingsState>,
    settings: Settings,
) -> Result<UpdateResult, String> {
    let settings = settings.sanitized();
    // Reject unparseable shortcuts up front instead of silently dropping them.
    hotkeys::validate(&settings.shortcuts)?;

    let previous = state.get();
    *state.current.lock().unwrap() = settings.clone();
    let mut warnings = Vec::new();
    if let Err(err) = state.save(&settings) {
        warnings.push(format!("Settings could not be saved: {err}"));
    }
    if previous.shortcuts != settings.shortcuts
        || previous.media_source != settings.media_source
        || previous.autostart != settings.autostart
    {
        warnings.extend(apply(&app, &settings));
    }
    let _ = app.emit(EVENT_SETTINGS, &settings);
    Ok(UpdateResult { settings, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_json_gives_defaults() {
        let s: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(s.migrated(), Settings::default());
    }

    #[test]
    fn migrates_v1_defaults_to_preset_values() {
        // The settings.json written by the Tahap 5 build (text size changed to 70%).
        let v1 = r##"{"visibleLines":3,"textScale":0.7,"align":"center","textOpacity":1.0,
            "backgroundOpacity":0.0,"activeColor":"#ffffff","animations":true,
            "hidePausedAfterMs":3000,"hideNoLyricsAfterMs":6000,"mediaSource":"auto",
            "globalOffsetMs":0,"offsetStepMs":250,"autostart":false}"##;
        let s = serde_json::from_str::<Settings>(v1).unwrap().migrated();
        assert_eq!(s.version, SETTINGS_VERSION);
        assert_eq!(s.preset, Preset::Minimal);
        assert_eq!(s.text_scale, 0.7);
        assert_eq!(s.active_color, None);
        assert_eq!(s.text_opacity, None);
        assert_eq!(s.background_opacity, None);
        assert_eq!(s.hide_paused_after_ms, Some(10_000));
    }

    #[test]
    fn migration_keeps_values_the_user_changed() {
        let v1 = r##"{"activeColor":"#F8C153","backgroundOpacity":0.5,"textOpacity":0.8,"hidePausedAfterMs":null}"##;
        let s = serde_json::from_str::<Settings>(v1).unwrap().migrated();
        assert_eq!(s.active_color.as_deref(), Some("#F8C153"));
        assert_eq!(s.background_opacity, Some(0.5));
        assert_eq!(s.text_opacity, Some(0.8));
        assert_eq!(s.hide_paused_after_ms, None);
    }

    #[test]
    fn current_files_are_not_migrated_again() {
        let v2 = r##"{"version":2,"activeColor":"#ffffff","hidePausedAfterMs":3000}"##;
        let s = serde_json::from_str::<Settings>(v2).unwrap().migrated();
        assert_eq!(s.active_color.as_deref(), Some("#ffffff"));
        assert_eq!(s.hide_paused_after_ms, Some(3000));
    }

    #[test]
    fn partial_json_keeps_other_defaults() {
        let s: Settings = serde_json::from_str(r#"{"visibleLines":1,"shortcuts":{"toggleEdit":"Ctrl+Shift+K"}}"#).unwrap();
        assert_eq!(s.visible_lines, 1);
        assert_eq!(s.shortcuts.toggle_edit, "Ctrl+Shift+K");
        assert_eq!(s.shortcuts.toggle_visible, "Ctrl+Alt+H");
        assert_eq!(s.align, Align::Center);
    }

    #[test]
    fn sanitize_clamps_out_of_range_values() {
        let s = Settings {
            visible_lines: 9,
            text_scale: f32::NAN,
            text_opacity: Some(0.0),
            background_opacity: Some(5.0),
            active_color: Some("red; background: url(x)".into()),
            global_offset_ms: 999_999,
            offset_step_ms: 0,
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(s.visible_lines, 3);
        assert_eq!(s.text_scale, 1.0);
        assert_eq!(s.text_opacity, Some(0.2));
        assert_eq!(s.background_opacity, Some(0.9));
        assert_eq!(s.active_color, None);
        assert_eq!(s.global_offset_ms, MAX_OFFSET_MS);
        assert_eq!(s.offset_step_ms, 50);
    }

    #[test]
    fn accepts_short_and_long_hex_colors() {
        assert!(is_hex_color("#fff"));
        assert!(is_hex_color("#FFD166"));
        assert!(!is_hex_color("fff"));
        assert!(!is_hex_color("#ggg"));
    }

    #[test]
    fn serializes_enums_as_lowercase() {
        let json = serde_json::to_value(Settings::default()).unwrap();
        assert_eq!(json["align"], "center");
        assert_eq!(json["preset"], "minimal");
        assert_eq!(json["activeColor"], serde_json::Value::Null);
        assert_eq!(json["mediaSource"], "auto");
        assert_eq!(json["shortcuts"]["offsetEarlier"], "Ctrl+Alt+Equal");
    }
}
