import { useCallback, useEffect, useRef, useState, type DragEvent, type FormEvent } from "react";
import {
  applyManualLyrics,
  getSearchDefaults,
  refreshLyrics,
  searchLyrics,
  type SearchResult,
} from "../lib/ipc";
import {
  fileMatchInfo,
  lyricsFromFile,
  lyricsFromResult,
  previewText,
  rankResults,
  resultMatchInfo,
} from "../lib/manualLyrics";
import { formatTime } from "../lib/nowPlaying";
import { useLyrics } from "../lib/useLyrics";
import { useNowPlaying } from "../lib/useNowPlaying";

type Status =
  | { kind: "idle" }
  | { kind: "searching" }
  | { kind: "error"; message: string }
  | { kind: "done"; message: string };

/** Duration difference in seconds between a hit and the playing track, if both are known. */
function durationDiff(result: SearchResult, playingMs: number): number | null {
  if (result.durationSecs === null || playingMs <= 0) return null;
  return Math.round(result.durationSecs - playingMs / 1000);
}

function diffClass(diff: number): string {
  const abs = Math.abs(diff);
  if (abs <= 3) return "diff close";
  if (abs <= 20) return "diff near";
  return "diff far";
}

export function SearchApp() {
  const { track } = useNowPlaying();
  const lyrics = useLyrics();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [status, setStatus] = useState<Status>({ kind: "idle" });
  const [dragging, setDragging] = useState(false);
  const fileInput = useRef<HTMLInputElement>(null);
  // The user typed their own query; stop replacing it with the playing track.
  const edited = useRef(false);
  const lastAutoQuery = useRef("");

  // Read inside runSearch without making it depend on (and re-run with) every snapshot.
  const playingMsRef = useRef(0);
  playingMsRef.current = track?.durationMs ?? 0;

  const runSearch = useCallback(async (text: string) => {
    if (!text.trim()) return;
    setStatus({ kind: "searching" });
    try {
      const found = rankResults(await searchLyrics(text), playingMsRef.current);
      setResults(found);
      setSelectedId(found[0]?.id ?? null);
      setStatus({ kind: "idle" });
    } catch (err) {
      setResults([]);
      setStatus({ kind: "error", message: `Search failed: ${err}` });
    }
  }, []);

  // Prefill with the playing track whenever the window gains focus (it stays alive hidden).
  useEffect(() => {
    const prefill = () => {
      if (edited.current) return;
      getSearchDefaults().then((defaults) => {
        if (!defaults || defaults.query === lastAutoQuery.current) return;
        lastAutoQuery.current = defaults.query;
        setQuery(defaults.query);
        runSearch(defaults.query);
      });
    };
    prefill();
    window.addEventListener("focus", prefill);
    return () => window.removeEventListener("focus", prefill);
  }, [runSearch]);

  const selected = results.find((r) => r.id === selectedId) ?? null;
  const selectedLyrics = selected ? lyricsFromResult(selected) : null;
  const playingMs = track?.durationMs ?? 0;

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    edited.current = true;
    runSearch(query);
  };

  const applySelected = async () => {
    if (!selected || !selectedLyrics) return;
    try {
      await applyManualLyrics(selectedLyrics, resultMatchInfo(selected));
      setStatus({ kind: "done", message: `Using "${selected.trackName}" by ${selected.artistName}.` });
    } catch (err) {
      setStatus({ kind: "error", message: String(err) });
    }
  };

  const importFile = async (file: File) => {
    try {
      const imported = lyricsFromFile(await file.text());
      await applyManualLyrics(imported, fileMatchInfo(file.name));
      const kind = imported.status === "synced" ? "synced" : "unsynced";
      setStatus({ kind: "done", message: `Imported ${file.name} (${kind}).` });
    } catch (err) {
      setStatus({ kind: "error", message: err instanceof Error ? err.message : String(err) });
    }
  };

  const resetToAutomatic = async () => {
    try {
      await refreshLyrics();
      setStatus({ kind: "done", message: "Looking up lyrics automatically again." });
    } catch (err) {
      setStatus({ kind: "error", message: String(err) });
    }
  };

  const onDrop = (e: DragEvent) => {
    e.preventDefault();
    setDragging(false);
    const file = e.dataTransfer.files[0];
    if (file) importFile(file);
  };

  const source = lyrics?.matched
    ? `${lyrics.matched.source === "file" ? "File" : "LRCLIB"}: ${lyrics.matched.trackName}${
        lyrics.matched.artistName ? ` — ${lyrics.matched.artistName}` : ""
      }`
    : lyrics
      ? "No lyrics in use"
      : null;

  return (
    <main
      className={dragging ? "search dragging" : "search"}
      onDragOver={(e) => {
        e.preventDefault();
        setDragging(true);
      }}
      onDragLeave={(e) => {
        if (e.currentTarget === e.target) setDragging(false);
      }}
      onDrop={onDrop}
    >
      <header className="now-playing">
        <div>
          <div className="np-label">Now playing</div>
          {track ? (
            <div className="np-track">
              {track.title} {track.artist && <span className="muted">— {track.artist}</span>}
              {track.durationMs > 0 && <span className="muted"> · {formatTime(track.durationMs)}</span>}
            </div>
          ) : (
            <div className="np-track muted">Nothing is playing</div>
          )}
          {source && (
            <div className="np-source muted">
              Lyrics in use: {source}
            </div>
          )}
        </div>
        <button type="button" disabled={!track} onClick={resetToAutomatic}>
          Reset to automatic
        </button>
      </header>

      <form className="query" onSubmit={onSubmit}>
        <input
          type="search"
          value={query}
          placeholder="Artist and title"
          onChange={(e) => {
            edited.current = true;
            setQuery(e.target.value);
          }}
          autoFocus
        />
        <button type="submit" className="primary" disabled={status.kind === "searching" || !query.trim()}>
          {status.kind === "searching" ? "Searching…" : "Search"}
        </button>
      </form>

      {status.kind === "error" && <div className="banner error">{status.message}</div>}
      {status.kind === "done" && <div className="banner info">{status.message}</div>}

      <div className="panes">
        <ul className="results" role="listbox" aria-label="Search results">
          {results.length === 0 && (
            <li className="empty muted">
              {status.kind === "searching" ? "Searching LRCLIB…" : "No results yet. Search by artist and title."}
            </li>
          )}
          {results.map((r) => {
            const diff = durationDiff(r, playingMs);
            return (
              <li
                key={r.id}
                role="option"
                aria-selected={r.id === selectedId}
                className={r.id === selectedId ? "result selected" : "result"}
                onClick={() => setSelectedId(r.id)}
              >
                <div className="result-title">{r.trackName}</div>
                <div className="result-meta muted">
                  {r.artistName}
                  {r.albumName && ` · ${r.albumName}`}
                </div>
                <div className="result-tags">
                  {r.instrumental ? (
                    <span className="tag">Instrumental</span>
                  ) : r.syncedLyrics ? (
                    <span className="tag synced">Synced</span>
                  ) : r.plainLyrics ? (
                    <span className="tag">Text only</span>
                  ) : (
                    <span className="tag">No lyrics</span>
                  )}
                  {r.durationSecs !== null && <span className="muted">{formatTime(r.durationSecs * 1000)}</span>}
                  {diff !== null && (
                    <span className={diffClass(diff)} title="Difference from the playing track">
                      {diff === 0 ? "exact" : `${diff > 0 ? "+" : ""}${diff} s`}
                    </span>
                  )}
                </div>
              </li>
            );
          })}
        </ul>

        <section className="preview">
          {selectedLyrics ? (
            <>
              <pre>{previewText(selectedLyrics)}</pre>
              <button type="button" className="primary" disabled={!track} onClick={applySelected}>
                Use these lyrics
              </button>
            </>
          ) : (
            <p className="muted">{selected ? "This entry has no lyrics." : "Select a result to preview it."}</p>
          )}
        </section>
      </div>

      <footer className="dropzone">
        Have your own lyrics? Drop an .lrc or .txt file here, or{" "}
        <button type="button" className="link" disabled={!track} onClick={() => fileInput.current?.click()}>
          browse…
        </button>
        <input
          ref={fileInput}
          type="file"
          accept=".lrc,.txt"
          hidden
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) importFile(file);
            e.target.value = "";
          }}
        />
      </footer>
    </main>
  );
}
