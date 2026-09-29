import { useEffect, useState } from "react";
import type { NowPlaying } from "../lib/ipc";
import { formatTime } from "../lib/nowPlaying";
import { positionAt, type PlaybackAnchor } from "../lib/syncClock";

interface TrackInfoProps {
  track: NowPlaying;
  anchor: PlaybackAnchor;
  /** Status line such as "Mencari lirik…". */
  note?: string;
}

/** Title + artist + position, shown when there are no lyrics to display. */
export function TrackInfo({ track, anchor, note }: TrackInfoProps) {
  const [now, setNow] = useState(() => performance.now());

  // Tick once per second only while playing.
  useEffect(() => {
    setNow(performance.now());
    if (!anchor.playing) return;
    const timer = window.setInterval(() => setNow(performance.now()), 1000);
    return () => window.clearInterval(timer);
  }, [anchor]);

  const raw = positionAt(anchor, now);
  const position = track.durationMs > 0 ? Math.min(raw, track.durationMs) : raw;
  const status = anchor.playing ? "▶" : "⏸";
  const time = track.durationMs > 0
    ? `${formatTime(position)} / ${formatTime(track.durationMs)}`
    : formatTime(position);

  return (
    <div className="lyrics">
      <p key={track.title} className="line active">
        {track.title || "(tanpa judul)"}
      </p>
      <p className="line dim">
        {[track.artist, `${status} ${time}`].filter(Boolean).join("  ·  ")}
      </p>
      {note && <p className="line dim">{note}</p>}
    </div>
  );
}
