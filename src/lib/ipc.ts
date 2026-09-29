import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Keep in sync with src-tauri/src/overlay.rs and src-tauri/src/media/mod.rs.
export const EVENT_EDIT_MODE = "overlay://edit-mode";
export const EVENT_NOW_PLAYING = "media://now-playing";

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

export function getEditMode(): Promise<boolean> {
  return invoke<boolean>("get_edit_mode");
}

export function onEditMode(handler: (edit: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>(EVENT_EDIT_MODE, (event) => handler(event.payload));
}

export function getNowPlaying(): Promise<NowPlaying | null> {
  return invoke<NowPlaying | null>("get_now_playing");
}

export function onNowPlaying(handler: (np: NowPlaying | null) => void): Promise<UnlistenFn> {
  return listen<NowPlaying | null>(EVENT_NOW_PLAYING, (event) => handler(event.payload));
}
