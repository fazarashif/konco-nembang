import "@fontsource-variable/atkinson-hyperlegible-next";
import React from "react";
import ReactDOM from "react-dom/client";
import { followSystemTheme } from "../lib/theme";
import "../settings/settings.css";
import "../styles/kn-tokens.css";
import { SearchApp } from "./SearchApp";
import "./search.css";

followSystemTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <SearchApp />
  </React.StrictMode>,
);
