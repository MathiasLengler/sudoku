import * as React from "react";
import * as ReactDOM from "react-dom";
import { App } from "./app/app";
import "../res/styles.css";

// Dokku deployment hacks
import "../res/mime.types";
import "../res/.static";

ReactDOM.render(
    <React.StrictMode>
        <App />
    </React.StrictMode>,
    document.getElementById("root")
);

if ("serviceWorker" in navigator && process.env.NODE_ENV === "production") {
    window.addEventListener("load", () => {
        navigator.serviceWorker
            .register("service-worker.js")
            .then(registration => {
                console.debug("SW registered: ", registration);
            })
            .catch(registrationError => {
                console.error("SW registration failed: ", registrationError);
            });
    });
}
