import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Keep in sync with the Rust side (overlay.rs, media/mod.rs, lyrics/mod.rs, settings.rs).
export const EVENT_EDIT_MODE = "overlay://edit-mode";
export const EVENT_TOAST = "overlay://toast";
export const EVENT_NOW_PLAYING = "media://now-playing";
export const EVENT_LYRICS = "lyrics://changed";
export const EVENT_SETTINGS = "settings://changed";

/** Mirrors `media::NowPlaying` in Rust. */
export interface NowPlaying {
  sourceApp: string;
  title: string;
  artist: string;
  album: string;
  durationMs: number;
  /** Position at `positionUpdatedAtMs`; extrapolate with `anchorFromNowPlaying`. */
  positionMs: number;
  /** Unix epoch ms when `positionMs` was valid; 0 if unknown. */
  positionUpdatedAtMs: number;
  playing: boolean;
  rate: number;
}

/** Mirrors `lyrics::MatchInfo` in Rust. */
export interface MatchInfo {
  source: string;
  id: number;
  trackName: string;
  artistName: string;
  durationSecs: number | null;
}

/** Mirrors `lyrics::Lyrics` in Rust (serde internally tagged on `status`). */
export type Lyrics =
  | { status: "loading" }
  | { status: "synced"; lrc: string }
  | { status: "plain"; text: string }
  | { status: "instrumental" }
  | { status: "notFound" }
  | { status: "error"; message: string };

/** Mirrors `lyrics::LyricsPayload`; `title`/`artist` are the raw metadata it belongs to. */
export type LyricsPayload = Lyrics & {
  title: string;
  artist: string;
  matched: MatchInfo | null;
  /** Per-track offset; positive shows lyrics earlier. */
  offsetMs: number;
};

export type Align = "left" | "center" | "right";
export type Preset = "minimal" | "pill" | "karaoke" | "subtle";
export type MediaSourceFilter = "auto" | "spotify" | "browser";

/** Mirrors `settings::Shortcuts`; accelerator strings such as `Ctrl+Alt+L`, empty = unbound. */
export interface Shortcuts {
  toggleEdit: string;
  toggleVisible: string;
  openSettings: string;
  offsetEarlier: string;
  offsetLater: string;
  offsetReset: string;
}

/** Mirrors `settings::Settings`. */
export interface Settings {
  version: number;
  preset: Preset;
  visibleLines: 1 | 2 | 3;
  textScale: number;
  align: Align;
  /** The next three override the preset; null keeps the preset's own value. */
  textOpacity: number | null;
  backgroundOpacity: number | null;
  activeColor: string | null;
  animations: boolean;
  hidePausedAfterMs: number | null;
  hideNoLyricsAfterMs: number | null;
  mediaSource: MediaSourceFilter;
  globalOffsetMs: number;
  offsetStepMs: number;
  shortcuts: Shortcuts;
  autostart: boolean;
}

export interface UpdateResult {
  settings: Settings;
  warnings: string[];
}

function subscribe<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, (e) => handler(e.payload));
}

export const getEditMode = () => invoke<boolean>("get_edit_mode");
export const onEditMode = (h: (edit: boolean) => void) => subscribe(EVENT_EDIT_MODE, h);
export const onToast = (h: (message: string) => void) => subscribe(EVENT_TOAST, h);
export const resetOverlayPosition = () => invoke<void>("reset_overlay_position");

export const getNowPlaying = () => invoke<NowPlaying | null>("get_now_playing");
export const onNowPlaying = (h: (np: NowPlaying | null) => void) => subscribe(EVENT_NOW_PLAYING, h);

export const getLyrics = () => invoke<LyricsPayload | null>("get_lyrics");
export const onLyrics = (h: (lyrics: LyricsPayload | null) => void) => subscribe(EVENT_LYRICS, h);
/** Nudge the current track's offset; null resets it. */
export const adjustTrackOffset = (deltaMs: number | null) => invoke<void>("adjust_track_offset", { deltaMs });

export const getSettings = () => invoke<Settings>("get_settings");
/** Rejects with a message when a shortcut is invalid or assigned twice. */
export const updateSettings = (settings: Settings) => invoke<UpdateResult>("update_settings", { settings });
export const onSettings = (h: (settings: Settings) => void) => subscribe(EVENT_SETTINGS, h);

/** Mirrors `lyrics::SearchResult`. */
export interface SearchResult {
  id: number;
  trackName: string;
  artistName: string;
  albumName: string | null;
  durationSecs: number | null;
  instrumental: boolean;
  syncedLyrics: string | null;
  plainLyrics: string | null;
}

export interface SearchDefaults {
  query: string;
  durationSecs: number | null;
}

/** Lyrics that can be pinned to a track (not loading/error). */
export type FinalLyrics = Exclude<Lyrics, { status: "loading" } | { status: "error" }>;

export const openSearch = () => invoke<void>("open_search");
export const getSearchDefaults = () => invoke<SearchDefaults | null>("get_search_defaults");
export const searchLyrics = (query: string) => invoke<SearchResult[]>("search_lyrics", { query });
/** Rejects with a message when nothing is playing. */
export const applyManualLyrics = (lyrics: FinalLyrics, matched: MatchInfo | null) =>
  invoke<void>("apply_manual_lyrics", { lyrics, matched });
/** Forget pinned/cached lyrics for the current track and look them up again. */
export const refreshLyrics = () => invoke<void>("refresh_lyrics");

export const EVENT_UPDATE = "update://available";

/** Mirrors `updates::UpdateInfo`. */
export interface UpdateInfo {
  version: string;
  currentVersion: string;
  notes: string | null;
}

export const checkForUpdate = () => invoke<UpdateInfo | null>("check_for_update");
/** An update the startup check already found, if any. */
export const pendingUpdate = () => invoke<UpdateInfo | null>("pending_update");
/** Downloads, verifies and installs the update found by the last check; the app restarts. */
export const installUpdate = () => invoke<void>("install_update");
export const onUpdateAvailable = (h: (info: UpdateInfo) => void) => subscribe(EVENT_UPDATE, h);

export const suspendHotkeys = () => invoke<void>("suspend_hotkeys");
export const resumeHotkeys = () => invoke<string[]>("resume_hotkeys");
