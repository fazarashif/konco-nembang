import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Keep in sync with src-tauri/src/overlay.rs.
export const EVENT_EDIT_MODE = "overlay://edit-mode";

export function getEditMode(): Promise<boolean> {
  return invoke<boolean>("get_edit_mode");
}

export function onEditMode(handler: (edit: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>(EVENT_EDIT_MODE, (event) => handler(event.payload));
}
