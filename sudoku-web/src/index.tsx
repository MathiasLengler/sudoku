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

if ("serviceWorker" in navigator && (process.env.NODE_ENV === "production" || process.env.DEBUG_SW)) {
    const { Workbox } = await import("workbox-window");

    const wb = new Workbox("service-worker.js");
    wb.addEventListener("waiting", event => {
        console.log(
            "A new service worker has installed, but it can't activate until all tabs running the current version have fully unloaded.",
            event
        );
    });
    wb.addEventListener("message", event => {
        if (event.data.type === "CACHE_UPDATED") {
            const { updatedURL } = event.data.payload;

            console.log(`A newer version of ${updatedURL} is available!`);
        }
    });

    // Add an event listener to detect when the registered
    // service worker has installed but is waiting to activate.
    wb.addEventListener("waiting", event => {
        (async () => {
            // Assuming the user accepted the update, set up a listener
            // that will reload the page as soon as the previously waiting
            // service worker has taken control.
            wb.addEventListener("controlling", () => {
                // At this point, reloading will ensure that the current
                // tab is loaded under the control of the new service worker.
                // Depending on your web app, you may want to auto-save or
                // persist transient state before triggering the reload.
                window.location.reload();
            });

            // When `event.wasWaitingBeforeRegister` is true, a previously
            // updated service worker is still waiting.
            // You may want to customize the UI prompt accordingly.

            // This code assumes your app has a promptForUpdate() method,
            // which returns true if the user wants to update.
            // Implementing this is app-specific; some examples are:
            // https://open-ui.org/components/alert.research or
            // https://open-ui.org/components/toast.research
            await new Promise(resolve => {
                console.log("Fake wait for accept");
                setTimeout(resolve, 3000);
            });
            console.log("Accepted");

            wb.messageSkipWaiting();
        })();
    });

    wb.register().catch(err => console.error("Workbox failed to register SW:", err));
}
