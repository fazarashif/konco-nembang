import "@fontsource-variable/atkinson-hyperlegible-next";
import React from "react";
import ReactDOM from "react-dom/client";
import "../styles/kn-tokens.css";
import { OverlayApp } from "./OverlayApp";
import "./overlay.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <OverlayApp />
  </React.StrictMode>,
);
