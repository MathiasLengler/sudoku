import * as Comlink from "comlink";
import { WasmCellWorld, WasmSudoku } from "../../../../../../sudoku-wasm/pkg";

import { WORKER_BOOT_UP_MESSAGE } from "../../../../constants";
import { init } from "./init";

if (process.env.NODE_ENV !== "production") {
    self.addEventListener("message", (ev) => {
        console.debug("Worker message RX:", ev.data);
    });
}

export type WorkerApi = {
    init: typeof init;
    // expose class constructors directly
    // Reference: https://github.com/GoogleChromeLabs/comlink/tree/main/docs/examples/03-classes-example
    WasmSudoku: typeof WasmSudoku;
    WasmCellWorld: typeof WasmCellWorld;
};

const workerApi: WorkerApi = {
    init,
    WasmSudoku,
    WasmCellWorld,
};

type Newable<T> = new (...args: unknown[]) => T;

function markClassAsProxy<T>(cls: Newable<T>) {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
    cls.prototype[Comlink.proxyMarker] = true;
}

// Ensure wasm-bindgen class instances are proxied instead of serialized
markClassAsProxy(WasmSudoku);
markClassAsProxy(WasmCellWorld);

// Use declaration merging (Module Augmentation) to reflect this modification.
// This corrects the inferred type of `Comlink.Remote`
declare module "../../../../../../sudoku-wasm/pkg" {
    // Declaration merging of classes only works with `interface`
    /* eslint-disable @typescript-eslint/consistent-type-definitions */
    interface WasmSudoku {
        [Comlink.proxyMarker]: true;
    }
    interface WasmCellWorld {
        [Comlink.proxyMarker]: true;
    }
    /* eslint-enable @typescript-eslint/consistent-type-definitions */
}

// Send boot up message
// Background: worker.tsx is an async module. (TODO: is this still the case?)
// This requires manual synchronization between Comlink.wrap and Comlink.expose,
// otherwise initialization messages from comlink would get lost, resulting in a deadlock.
postMessage(WORKER_BOOT_UP_MESSAGE);

Comlink.expose(workerApi);
