import * as Comlink from "comlink";
import { WasmCellWorld, WasmSudoku } from "sudoku-wasm";
import type {
    DynamicGeneratorSettings,
    DynamicMultiShotGeneratorSettings,
    DynamicSolveStep,
    GeneratorProgress,
    MultiShotGeneratorProgress,
    StrategyEnums,
} from "../../../../types";
import type { SerializedDynamicCellWorld, SerializedDynamicSudoku } from "../../../utils/serializedData";
import { init } from "./init";

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

/**
 * Result of an expensive operation that returns both result and updated sudoku state.
 */
export type ExpensiveOperationResult<T> = {
    serializedSudoku: SerializedDynamicSudoku;
    result: T;
};

/**
 * Expensive operations API that receives serialized state, executes operation,
 * and returns serialized result. This avoids keeping WasmSudoku instances on the worker.
 */
const expensiveOperations = {
    /**
     * Generate a new sudoku grid.
     */
    generate(
        settings: DynamicGeneratorSettings,
        onProgress: (progress: GeneratorProgress) => void,
    ): SerializedDynamicSudoku {
        console.debug("Worker: generate", settings);
        const wasmSudoku = WasmSudoku.generate(settings, onProgress);
        const serialized = wasmSudoku.serialize();
        wasmSudoku.free();
        return Comlink.transfer(serialized, [serialized.buffer]);
    },

    /**
     * Generate a new sudoku grid using multi-shot generation.
     */
    generateMultiShot(
        settings: DynamicMultiShotGeneratorSettings,
        onProgress: (progress: MultiShotGeneratorProgress) => void,
    ): SerializedDynamicSudoku {
        console.debug("Worker: generateMultiShot", settings);
        const wasmSudoku = WasmSudoku.generateMultiShot(settings, onProgress);
        const serialized = wasmSudoku.serialize();
        wasmSudoku.free();
        return Comlink.transfer(serialized, [serialized.buffer]);
    },

    /**
     * Try strategies on the sudoku.
     * Receives serialized sudoku, applies strategies, returns updated state and result.
     */
    tryStrategies(
        serializedSudoku: SerializedDynamicSudoku,
        strategies: StrategyEnums,
    ): ExpensiveOperationResult<DynamicSolveStep | undefined> {
        console.debug("Worker: tryStrategies", strategies);
        const wasmSudoku = WasmSudoku.deserialize(serializedSudoku);
        const result = wasmSudoku.tryStrategies(strategies) ?? undefined;
        const serialized = wasmSudoku.serialize();
        wasmSudoku.free();
        return {
            serializedSudoku: Comlink.transfer(serialized, [serialized.buffer]),
            result,
        };
    },

    /**
     * Import a sudoku from a string. This is an expensive operation since it
     * needs to solve the grid to determine its properties.
     */
    importSudoku(input: string, setAllDirectCandidates: boolean): SerializedDynamicSudoku {
        console.debug("Worker: importSudoku");
        const wasmSudoku = WasmSudoku.import(input);
        if (setAllDirectCandidates) {
            wasmSudoku.setAllDirectCandidates();
        }
        const serialized = wasmSudoku.serialize();
        wasmSudoku.free();
        return Comlink.transfer(serialized, [serialized.buffer]);
    },
};

/**
 * Type for expensive operations that are called via Comlink.
 * All methods return Promises because Comlink promisifies all calls.
 */
export type ExpensiveOperationsApi = {
    generate: (
        settings: DynamicGeneratorSettings,
        onProgress: (progress: GeneratorProgress) => void,
    ) => Promise<SerializedDynamicSudoku>;

    generateMultiShot: (
        settings: DynamicMultiShotGeneratorSettings,
        onProgress: (progress: MultiShotGeneratorProgress) => void,
    ) => Promise<SerializedDynamicSudoku>;

    tryStrategies: (
        serializedSudoku: SerializedDynamicSudoku,
        strategies: StrategyEnums,
    ) => Promise<ExpensiveOperationResult<DynamicSolveStep | undefined>>;

    importSudoku: (input: string, setAllDirectCandidates: boolean) => Promise<SerializedDynamicSudoku>;
};

// Cast to satisfy the type (Comlink will promisify these)
const expensiveOperationsImpl: ExpensiveOperationsApi = expensiveOperations as unknown as ExpensiveOperationsApi;

export type WorkerApi = {
    init: typeof init;
    // expose class constructors directly
    // Reference: https://github.com/GoogleChromeLabs/comlink/tree/main/docs/examples/03-classes-example
    WasmSudoku: typeof WasmSudoku;
    WasmSudokuWithTransfer: typeof WasmSudokuWithTransfer;
    WasmCellWorld: typeof WasmCellWorld;
    WasmCellWorldWithTransfer: typeof WasmCellWorldWithTransfer;
    // Expensive operations API for stateless operations
    expensiveOperations: ExpensiveOperationsApi;
};

const workerApi: WorkerApi = {
    init,
    WasmSudoku,
    WasmSudokuWithTransfer,
    WasmCellWorld,
    WasmCellWorldWithTransfer,
    expensiveOperations: expensiveOperationsImpl,
};

// The type of `obj` ensures that only module-augmented classed can be patched with the marker.
function markObjectAsComlinkProxy(obj: { prototype: { [Comlink.proxyMarker]: true } }) {
    obj.prototype[Comlink.proxyMarker] = true;
}

// Ensure wasm-bindgen class instances are proxied instead of serialized
markObjectAsComlinkProxy(WasmSudoku);
markObjectAsComlinkProxy(WasmCellWorld);

// Mark expensiveOperations object as a Comlink proxy so its methods can be called remotely
(expensiveOperations as Record<symbol, boolean>)[Comlink.proxyMarker] = true;

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
