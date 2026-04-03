export function spawnWorker() {
    console.debug("Spawning worker");
    const worker = new Worker(new URL("./bg/worker.tsx", import.meta.url), {
        name: "SudokuWasmWorker",
        type: "module",
    });
    if (import.meta.env.MODE === "development") {
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
    return worker;
}
