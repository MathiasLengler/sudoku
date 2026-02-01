import * as Comlink from "comlink";
import { atom, type Getter } from "jotai";
import { atomWithDefault } from "jotai/utils";
import { WasmSudoku as WasmSudokuMaybeUninit, type StrategySet, type WasmSudoku } from "sudoku-wasm";
import type { Asyncify, SetReturnType } from "type-fest";
import type { DynamicGrid } from "../../../types";
import { loadCells } from "../cellsPersistence";
import { GENERATE_FORM_DEFAULT_VALUES } from "../forms/generate";
import { initWasm } from "../wasm/init";
import { remoteWasmSudokuClassState, type UnsafeRemoteWasmSudoku } from "../worker";

type ExpensiveConstructorFunctions = "fromDynamicGrid" | "generate" | "generateMultiShot" | "import";
type ExpensiveMethods = "tryStrategies";

/* eslint-disable @typescript-eslint/no-explicit-any */
type MakeMainThreadWasmSudokuStatics<TWasmSudokuClass> = {
    [Property in keyof TWasmSudokuClass]: TWasmSudokuClass[Property] extends (...args: any) => any
        ? Property extends ExpensiveConstructorFunctions
            ? Asyncify<ReplaceWasmSudokuReturnWithMainThreadWasmSudoku<TWasmSudokuClass[Property]>>
            : ReplaceWasmSudokuReturnWithMainThreadWasmSudoku<TWasmSudokuClass[Property]>
        : never; // static fields are not supported
};
type MakeMainThreadWasmSudokuMethods<TWasmSudokuInstance> = {
    [Property in keyof TWasmSudokuInstance]: TWasmSudokuInstance[Property] extends (...arguments_: any[]) => any
        ? Property extends ExpensiveMethods
            ? Asyncify<TWasmSudokuInstance[Property]>
            : TWasmSudokuInstance[Property]
        : never; // instance fields are not supported
};
type ReplaceWasmSudokuReturnWithMainThreadWasmSudoku<T extends (...args: any) => any> =
    ReturnType<T> extends WasmSudoku ? SetReturnType<T, MainThreadWasmSudoku> : T;
/* eslint-enable @typescript-eslint/no-explicit-any */

export type MainThreadWasmSudokuClass = MakeMainThreadWasmSudokuStatics<typeof WasmSudoku>;
export type MainThreadWasmSudoku = MakeMainThreadWasmSudokuMethods<WasmSudoku>;

const wasmSudokuClassState = atom<Promise<typeof WasmSudoku>>(async () => {
    await initWasm();
    return WasmSudokuMaybeUninit;
});

export const mainThreadWasmSudokuClassState = atom<Promise<MainThreadWasmSudokuClass>>(async (get) => {
    const WasmSudoku = await get(wasmSudokuClassState);

    const moveRemoteWasmSudokuToMainThread = async (
        remoteWasmSudoku: UnsafeRemoteWasmSudoku,
    ): Promise<MainThreadWasmSudoku> => {
        const serializedWasmSudoku = await remoteWasmSudoku.serialize();
        const wasmSudoku = WasmSudoku.deserialize(serializedWasmSudoku);
        return createInstanceProxy(get, wasmSudoku);
    };
    // FIXME: this is a fake class
    //  are we able to define this as a real class, but retain the Proxy magic?
    //  the class would *not* extend WasmSudoku, since it overrides expensive methods incompatibly
    //  Current scope access would be replaced with constructor parameters (?)
    const MainThreadWasmSudoku = {
        prototype: undefined as never,

        // Cheap constructor functions
        new(base): MainThreadWasmSudoku {
            return createInstanceProxy(get, WasmSudoku.new(base));
        },
        deserialize(bytes): MainThreadWasmSudoku {
            return createInstanceProxy(get, WasmSudoku.deserialize(bytes));
        },

        // Expensive constructor functions
        async generate(generator_settings, on_progress): Promise<MainThreadWasmSudoku> {
            const RemoteWasmSudokuClass = await get(remoteWasmSudokuClassState);
            const remoteWasmSudoku = await RemoteWasmSudokuClass.generate(
                generator_settings,
                Comlink.proxy(on_progress),
            );
            return moveRemoteWasmSudokuToMainThread(remoteWasmSudoku);
        },
        async generateMultiShot(multi_shot_generator_settings, on_progress): Promise<MainThreadWasmSudoku> {
            const RemoteWasmSudokuClass = await get(remoteWasmSudokuClassState);
            const remoteWasmSudoku = await RemoteWasmSudokuClass.generateMultiShot(
                multi_shot_generator_settings,
                Comlink.proxy(on_progress),
            );
            return moveRemoteWasmSudokuToMainThread(remoteWasmSudoku);
        },
        async fromDynamicGrid(dynamic_grid): Promise<MainThreadWasmSudoku> {
            const RemoteWasmSudokuClass = await get(remoteWasmSudokuClassState);
            const remoteWasmSudoku = await RemoteWasmSudokuClass.fromDynamicGrid(dynamic_grid);
            return moveRemoteWasmSudokuToMainThread(remoteWasmSudoku);
        },
        async import(input): Promise<MainThreadWasmSudoku> {
            const RemoteWasmSudokuClass = await get(remoteWasmSudokuClassState);
            const remoteWasmSudoku = await RemoteWasmSudokuClass.import(input);
            return moveRemoteWasmSudokuToMainThread(remoteWasmSudoku);
        },
    } satisfies MainThreadWasmSudokuClass;

    return MainThreadWasmSudoku;
});

function createInstanceProxy(get: Getter, wasmSudoku: WasmSudoku): MainThreadWasmSudoku {
    // Clone current `wasmSudoku` instance into the worker.
    const moveWasmSudokuToWorker = async () => {
        const RemoteWasmSudokuClass = await get(remoteWasmSudokuClassState);
        const serializedDynamicSudoku = wasmSudoku.serialize();
        const remoteWasmSudoku = await RemoteWasmSudokuClass.deserialize(
            Comlink.transfer(serializedDynamicSudoku, [serializedDynamicSudoku.buffer]),
        );
        return remoteWasmSudoku;
    };

    const proxy = new Proxy(wasmSudoku, {
        get(wasmSudoku, prop: keyof WasmSudoku, receiver) {
            if (prop === "tryStrategies") {
                return (async (strategies: StrategySet) => {
                    const remoteWasmSudoku = await moveWasmSudokuToWorker();
                    const res = await remoteWasmSudoku.tryStrategies(strategies);
                    return res;
                }) satisfies Asyncify<WasmSudoku["tryStrategies"]>;
            }
            // Assert we have delegated all non-expensive methods
            prop satisfies Exclude<keyof WasmSudoku, ExpensiveMethods>;
            return Reflect.get(wasmSudoku, prop, receiver) as unknown;
        },
    });
    return proxy as unknown as MainThreadWasmSudoku;
}

async function createMainThreadWasmSudoku(
    MainThreadWasmSudoku: MainThreadWasmSudokuClass,
    dynamicGrid?: DynamicGrid,
): Promise<MainThreadWasmSudoku> {
    if (dynamicGrid) {
        console.debug("Restoring sudoku");
        try {
            return await MainThreadWasmSudoku.fromDynamicGrid(dynamicGrid);
        } catch (err) {
            console.error("Failed to restore persisted grid:", err);
        }
    }
    console.debug("Generating initial sudoku");
    return await MainThreadWasmSudoku.generate(
        {
            base: GENERATE_FORM_DEFAULT_VALUES.base,
            prune: {
                target: "minimal",
                strategies: GENERATE_FORM_DEFAULT_VALUES.strategies,
                setAllDirectCandidates: GENERATE_FORM_DEFAULT_VALUES.setAllDirectCandidates,
                order: "random",
                startFromNearMinimalGrid: false,
            },
        },
        (progress) => {
            console.debug("Initial sudoku generation progress:", progress);
        },
    );
}

export const wasmSudokuState = atomWithDefault<MainThreadWasmSudoku | Promise<MainThreadWasmSudoku>>(async (get) => {
    const MainThreadWasmSudokuClass = await get(mainThreadWasmSudokuClassState);
    const cells = loadCells();
    return await createMainThreadWasmSudoku(MainThreadWasmSudokuClass, cells);
});
