import React from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app/app";
import "../res/styles.css";

const container = document.getElementById("root");
if (!container) throw new Error("React root container not found");
const root = createRoot(container);
root.render(
    <React.StrictMode>
        <App />
    </React.StrictMode>
);
