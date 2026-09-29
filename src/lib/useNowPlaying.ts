import { useEffect, useState } from "react";
import { getNowPlaying, onNowPlaying, type NowPlaying } from "./ipc";
import { anchorFromNowPlaying } from "./nowPlaying";
import type { PlaybackAnchor } from "./syncClock";

export interface NowPlayingState {
  track: NowPlaying | null;
  /** Re-created on every backend snapshot (track change, play/pause, seek, resync). */
  anchor: PlaybackAnchor | null;
}

function toState(np: NowPlaying | null): NowPlayingState {
  return {
    track: np,
    anchor: np ? anchorFromNowPlaying(np, performance.now(), Date.now()) : null,
  };
}

export function useNowPlaying(): NowPlayingState {
  const [state, setState] = useState<NowPlayingState>({ track: null, anchor: null });

  useEffect(() => {
    let live = false;
    const unlisten = onNowPlaying((np) => {
      live = true;
      setState(toState(np));
    });
    // Covers the snapshot emitted before this page started listening.
    getNowPlaying().then((np) => {
      if (!live) setState(toState(np));
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return state;
}
