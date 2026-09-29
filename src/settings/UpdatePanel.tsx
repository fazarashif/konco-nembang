import { useEffect, useState } from "react";
import { checkForUpdate, installUpdate, onUpdateAvailable, pendingUpdate, type UpdateInfo } from "../lib/ipc";

type State =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "upToDate" }
  | { kind: "available"; info: UpdateInfo }
  | { kind: "installing"; info: UpdateInfo }
  | { kind: "error"; message: string };

export function UpdatePanel() {
  const [state, setState] = useState<State>({ kind: "idle" });

  // The startup check in the backend may already have found one, before or after this opened.
  useEffect(() => {
    pendingUpdate().then((info) => {
      if (info) setState({ kind: "available", info });
    });
    const unlisten = onUpdateAvailable((info) => setState({ kind: "available", info }));
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const check = async () => {
    setState({ kind: "checking" });
    try {
      const info = await checkForUpdate();
      setState(info ? { kind: "available", info } : { kind: "upToDate" });
    } catch (err) {
      setState({ kind: "error", message: `Couldn't check for updates: ${err}` });
    }
  };

  const install = async (info: UpdateInfo) => {
    setState({ kind: "installing", info });
    try {
      await installUpdate();
    } catch (err) {
      setState({ kind: "error", message: `Update failed: ${err}` });
    }
  };

  let status: string;
  switch (state.kind) {
    case "idle":
      status = "Updates are checked automatically when the app starts.";
      break;
    case "checking":
      status = "Checking…";
      break;
    case "upToDate":
      status = "You're on the latest version.";
      break;
    case "available":
      status = `Version ${state.info.version} is available.`;
      break;
    case "installing":
      status = `Downloading ${state.info.version}. The app will restart.`;
      break;
    case "error":
      status = state.message;
      break;
  }

  return (
    <div className="row">
      <div className="row-text">
        <span className="row-label">Updates</span>
        <span className="row-hint">{status}</span>
        {state.kind === "available" && state.info.notes && <span className="row-hint">{state.info.notes}</span>}
      </div>
      <div className="row-control">
        {state.kind === "available" ? (
          <button type="button" className="primary" onClick={() => install(state.info)}>
            Install and restart
          </button>
        ) : (
          <button
            type="button"
            disabled={state.kind === "checking" || state.kind === "installing"}
            onClick={check}
          >
            Check for updates
          </button>
        )}
      </div>
    </div>
  );
}
