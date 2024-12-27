import { useEffect, useState } from "react";
import { Alert, Snackbar } from "@mui/material";

const SW_ENABLED = "serviceWorker" in navigator && import.meta.env.VITE_SW_ENABLED === "true";

export function WorkboxManager() {
    const [showUpdateNotification, setShowUpdateNotification] = useState(false);

    useEffect(() => {
        async function initWorkbox() {
            const { Workbox } = await import("workbox-window");

            const wb = new Workbox("service-worker.js");
            wb.addEventListener("waiting", (event) => {
                console.log(
                    "A new service worker has installed, but it can't activate until all tabs running the current version have fully unloaded.",
                    event,
                );
            });

            // Add an event listener to detect when the registered
            // service worker has installed but is waiting to activate.
            wb.addEventListener("waiting", (_event) => {
                setShowUpdateNotification(true);

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

                wb.messageSkipWaiting();
            });

            wb.register().catch((err) => console.error("Workbox failed to register SW:", err));
        }

        if (SW_ENABLED) {
            console.info("Service Worker enabled");
            initWorkbox().catch((err) => console.error("Unexpected error while initializing Workbox:", err));
        } else {
            console.debug("Service Worker disabled");
        }
    }, []);

    const handleUpdateNotificationClose = () => setShowUpdateNotification(false);

    return (
        <>
            <Snackbar
                open={showUpdateNotification}
                onClose={handleUpdateNotificationClose}
                anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
            >
                <Alert onClose={handleUpdateNotificationClose} severity="info">
                    Updating...
                </Alert>
            </Snackbar>
        </>
    );
}
