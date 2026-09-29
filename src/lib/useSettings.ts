import { useEffect, useState } from "react";
import { getSettings, onSettings, type Settings } from "./ipc";

/** Current settings, kept live across windows. Null until the first load. */
export function useSettings(): Settings | null {
  const [settings, setSettings] = useState<Settings | null>(null);

  useEffect(() => {
    let live = false;
    const unlisten = onSettings((s) => {
      live = true;
      setSettings(s);
    });
    getSettings().then((s) => {
      if (!live) setSettings(s);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return settings;
}
