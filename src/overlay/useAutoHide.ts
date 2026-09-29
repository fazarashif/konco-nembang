import { useEffect, useState } from "react";
import { autoHideDelay, type AutoHideSettings, type OverlayCondition } from "../lib/autoHide";

/**
 * Whether the overlay should currently be faded out. `resetKey` (the track identity)
 * brings it back whenever a new song starts.
 */
export function useAutoHide(
  condition: OverlayCondition,
  settings: AutoHideSettings,
  resetKey: string,
): boolean {
  const delay = autoHideDelay(condition, settings);
  const [hidden, setHidden] = useState(delay === 0);

  useEffect(() => {
    if (delay === null) {
      setHidden(false);
      return;
    }
    if (delay === 0) {
      setHidden(true);
      return;
    }
    setHidden(false);
    const timer = window.setTimeout(() => setHidden(true), delay);
    return () => window.clearTimeout(timer);
  }, [delay, resetKey]);

  return hidden;
}
