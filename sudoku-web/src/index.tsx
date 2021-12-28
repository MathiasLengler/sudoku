import * as React from "react";
import * as ReactDOM from "react-dom";
import { App } from "./app/app";
import "../res/styles.css";
import "../res/mime.types";

ReactDOM.render(<App />, document.getElementById("root"));

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
