import { WORKER_BOOT_UP_MESSAGE } from "../../../constants";

export async function spawnWorker() {
    console.debug("Spawning worker");
    const worker = new Worker(new URL("./worker.tsx", import.meta.url), { name: "SudokuWasmWorker" });
    if (process.env.NODE_ENV !== "production") {
        console.debug("Attaching debug event listeners");
        worker.addEventListener("message", (ev) => {
            console.debug("Worker message TX:", ev.data);
        });
        worker.addEventListener("error", (ev) => {
            console.error("Worker error:", ev);
        });
        worker.addEventListener("messageerror", (ev) => {
            console.error("Worker messageerror:", ev);
        });
    }
    console.debug("Waiting for worker boot up message");
    const bootUpMessage = await new Promise((resolve, reject) => {
        worker.addEventListener(
            "message",
            (ev: MessageEvent) => {
                if (ev.data === WORKER_BOOT_UP_MESSAGE) {
                    resolve(ev.data);
                } else {
                    reject(new Error(`Unexpected message: ${ev.data}`));
                }
            },
            { once: true },
        );
    });
    console.debug("Received worker boot up message:", bootUpMessage);

    return worker;
}
