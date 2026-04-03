import * as Comlink from "comlink";
import { WasmCellWorld, WasmSudoku } from "sudoku-wasm";
import type { SerializedDynamicCellWorld, SerializedDynamicSudoku } from "../../../utils/serializedData";
import { initWasm } from "../../wasm/init";

if (import.meta.env.MODE === "development") {
    self.addEventListener("message", (ev) => {
        console.debug("Worker message RX:", ev.data);
    });
}

export class WasmSudokuWithTransfer extends WasmSudoku {
    static override deserialize(bytes: SerializedDynamicSudoku): WasmSudokuWithTransfer {
        const instance = WasmSudoku.deserialize(bytes);
        Object.setPrototypeOf(instance, this.prototype);
        return instance as WasmSudokuWithTransfer;
    }

    override serialize(): SerializedDynamicSudoku {
        const serialized = super.serialize();
        return Comlink.transfer(serialized, [serialized.buffer]);
    }
}

export class WasmCellWorldWithTransfer extends WasmCellWorld {
    static override deserialize(bytes: SerializedDynamicCellWorld): WasmCellWorldWithTransfer {
        const instance = WasmCellWorld.deserialize(bytes);
        Object.setPrototypeOf(instance, this.prototype);
        return instance as WasmCellWorldWithTransfer;
    }
    override serialize(): SerializedDynamicCellWorld {
        const serialized = super.serialize();
        return Comlink.transfer(serialized, [serialized.buffer]);
    }
}

export type WorkerApi = {
    init: typeof initWasm;
    // expose class constructors directly
    // Reference: https://github.com/GoogleChromeLabs/comlink/tree/main/docs/examples/03-classes-example
    WasmSudoku: typeof WasmSudoku;
    WasmSudokuWithTransfer: typeof WasmSudokuWithTransfer;
    WasmCellWorld: typeof WasmCellWorld;
    WasmCellWorldWithTransfer: typeof WasmCellWorldWithTransfer;
};

const workerApi: WorkerApi = {
    init: initWasm,
    WasmSudoku,
    WasmSudokuWithTransfer,
    WasmCellWorld,
    WasmCellWorldWithTransfer,
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

Comlink.expose(workerApi);
