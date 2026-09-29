import { useEffect, useState } from "react";
import { getLyrics, onLyrics, type LyricsPayload } from "../lib/ipc";

export function useLyrics(): LyricsPayload | null {
  const [lyrics, setLyrics] = useState<LyricsPayload | null>(null);

  useEffect(() => {
    let live = false;
    const unlisten = onLyrics((payload) => {
      live = true;
      setLyrics(payload);
    });
    // Covers the answer emitted before this page started listening.
    getLyrics().then((payload) => {
      if (!live) setLyrics(payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return lyrics;
}
