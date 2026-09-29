import { useEffect, useMemo, useState } from "react";
import { getEditMode, onEditMode } from "../lib/ipc";
import { parseLrc } from "../lib/lrc";
import type { PlaybackAnchor } from "../lib/syncClock";
import { DEMO_DURATION_MS, DEMO_LRC } from "./demoLyrics";
import { LyricsView } from "./LyricsView";
import { useActiveLine } from "./useActiveLine";

function startAnchor(): PlaybackAnchor {
  return { positionMs: 0, atMs: performance.now(), playing: true };
}

export function OverlayApp() {
  const lines = useMemo(() => parseLrc(DEMO_LRC).lines, []);
  const [anchor, setAnchor] = useState(startAnchor);
  const [editMode, setEditMode] = useState(false);
  const activeIndex = useActiveLine(lines, anchor);

  // Loop the demo track.
  useEffect(() => {
    const timer = window.setTimeout(() => setAnchor(startAnchor()), DEMO_DURATION_MS);
    return () => window.clearTimeout(timer);
  }, [anchor]);

  useEffect(() => {
    getEditMode().then(setEditMode);
    const unlisten = onEditMode(setEditMode);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className={editMode ? "frame edit" : "frame"} data-tauri-drag-region>
      {editMode && (
        <div className="badge" data-tauri-drag-region>
          MODE EDIT: drag untuk pindah, tarik tepi untuk resize. Ctrl+Alt+L untuk kunci.
        </div>
      )}
      <LyricsView lines={lines} activeIndex={activeIndex} visibleLines={3} />
    </div>
  );
}
