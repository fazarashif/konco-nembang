import { useEffect, useState } from "react";
import { onToast } from "../lib/ipc";

const TOAST_MS = 1600;

/** Brief feedback from the backend, e.g. after an offset hotkey. */
export function Toast() {
  const [toast, setToast] = useState<{ message: string; id: number } | null>(null);

  useEffect(() => {
    let id = 0;
    const unlisten = onToast((message) => setToast({ message, id: ++id }));
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), TOAST_MS);
    return () => window.clearTimeout(timer);
  }, [toast]);

  if (!toast) return null;
  return (
    <div key={toast.id} className="toast">
      {toast.message}
    </div>
  );
}
