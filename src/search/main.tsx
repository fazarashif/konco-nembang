import React from "react";
import ReactDOM from "react-dom/client";
import { SearchApp } from "./SearchApp";
import "../settings/settings.css";
import "./search.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <SearchApp />
  </React.StrictMode>,
);
