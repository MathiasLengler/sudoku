import * as Comlink from "comlink";
import { WasmCellWorld, WasmSudoku } from "sudoku-wasm";

import { WORKER_BOOT_UP_MESSAGE } from "../../../constants";
import { init } from "./init";
import type { SerializedDynamicCellWorld, SerializedDynamicSudoku } from "../../../utils/serializedData";

if (import.meta.env.MODE === "development") {
    self.addEventListener("message", (ev) => {
        console.debug("Worker message RX:", ev.data);
    });
}

class WasmSudokuWithTransfer extends WasmSudoku {
    static override deserialize(bytes: SerializedDynamicSudoku): WasmSudokuWithTransfer {
        const instance = WasmSudoku.deserialize(bytes);
        Object.setPrototypeOf(instance, this.prototype);
        return instance as WasmSudokuWithTransfer;
    }

    serializeWithTransfer(): SerializedDynamicSudoku {
        const serialized = super.serialize();
        return Comlink.transfer(serialized, [serialized.buffer]);
    }
}

class WasmCellWorldWithTransfer extends WasmCellWorld {
    static override deserialize(bytes: SerializedDynamicCellWorld): WasmCellWorldWithTransfer {
        const instance = WasmCellWorld.deserialize(bytes);
        Object.setPrototypeOf(instance, this.prototype);
        return instance as WasmCellWorldWithTransfer;
    }
    serializeWithTransfer(): SerializedDynamicCellWorld {
        const serialized = super.serialize();
        return Comlink.transfer(serialized, [serialized.buffer]);
    }
}
export type MicroBenchmarkAPI = {
    echoCloneUint8Array: (data: Uint8Array) => Uint8Array;
    echoTransferUint8Array: (data: Uint8Array) => Uint8Array;
};

export type WorkerApi = {
    init: typeof init;
    // expose class constructors directly
    // Reference: https://github.com/GoogleChromeLabs/comlink/tree/main/docs/examples/03-classes-example
    WasmSudoku: typeof WasmSudoku;
    WasmSudokuWithTransfer: typeof WasmSudokuWithTransfer;
    WasmCellWorld: typeof WasmCellWorld;
    WasmCellWorldWithTransfer: typeof WasmCellWorldWithTransfer;

    benchmark: MicroBenchmarkAPI;
};

const workerApi: WorkerApi = {
    init,
    WasmSudoku,
    WasmSudokuWithTransfer,
    WasmCellWorld,
    WasmCellWorldWithTransfer,
    benchmark: {
        echoCloneUint8Array: (data: Uint8Array) => {
            return data;
        },
        echoTransferUint8Array: (data: Uint8Array) => {
            return Comlink.transfer(data, [data.buffer]);
        },
    },
};

// The type of `obj` ensures that only module-augmented classed can be patched with the marker.
function markObjectAsComlinkProxy(obj: { prototype: { [Comlink.proxyMarker]: true } }) {
    obj.prototype[Comlink.proxyMarker] = true;
}

// Ensure wasm-bindgen class instances are proxied instead of serialized
markObjectAsComlinkProxy(WasmSudoku);
markObjectAsComlinkProxy(WasmCellWorld);

// Use declaration merging (Module Augmentation) to reflect this modification.
// This corrects the inferred type of `Comlink.Remote`
declare module "sudoku-wasm" {
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
