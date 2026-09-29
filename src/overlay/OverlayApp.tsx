import { useEffect, useState } from "react";
import { getEditMode, onEditMode } from "../lib/ipc";
import { TrackInfo } from "./TrackInfo";
import { useNowPlaying } from "./useNowPlaying";

export function OverlayApp() {
  const { track, anchor } = useNowPlaying();
  const [editMode, setEditMode] = useState(false);

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
      {track && anchor ? (
        <TrackInfo track={track} anchor={anchor} />
      ) : (
        // Stay invisible when nothing plays; only hint at it while positioning the overlay.
        editMode && (
          <div className="lyrics">
            <p className="line dim">Tidak ada lagu yang diputar</p>
          </div>
        )
      )}
    </div>
  );
}
