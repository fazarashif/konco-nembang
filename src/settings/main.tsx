import "@fontsource-variable/atkinson-hyperlegible-mono";
import "@fontsource-variable/atkinson-hyperlegible-next";
import React from "react";
import ReactDOM from "react-dom/client";
import { followSystemTheme } from "../lib/theme";
import "../styles/kn-tokens.css";
import { SettingsApp } from "./SettingsApp";
import "./settings.css";

followSystemTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <SettingsApp />
  </React.StrictMode>,
);
