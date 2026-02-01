import { WasmSudoku, type GenerateOnProgress } from "sudoku-wasm";
import type { Asyncify, SetReturnType } from "type-fest";
import type { DynamicGeneratorSettings, DynamicSolveStep, StrategySet } from "../../../types";
import { type RemoteWasmSudokuClass } from "../worker";

// FIXME: model from rust side
//  sudoku-wasm could export two different WasmSudoku classes: expensive/cheap


type ExpensiveMethods = "tryStrategies";
type ExpensiveConstructorFunctions = "generate" | "generateMultiShot" | "import";

type BaseMainThreadWasmSudoku = Omit<WasmSudoku, ExpensiveMethods>;

type DelegatedMethods = {
    tryStrategies: Asyncify<WasmSudoku["tryStrategies"]>;
};

type DelegatedConstructorFunctions = {
    generate: Asyncify<ReplaceWasmSudokuReturnWithMainThreadWasmSudoku<(typeof WasmSudoku)["generate"]>>;
};

type ReplaceWasmSudokuReturnWithMainThreadWasmSudoku<T extends (...args: any) => any> =
    ReturnType<T> extends WasmSudoku ? SetReturnType<T, MainThreadWasmSudoku> : T;

export type MainThreadWasmSudoku = BaseMainThreadWasmSudoku & DelegatedMethods;

// TODO: assert that all ExpensiveConstructorFunctions are covered
export const MainThreadWasmSudoku = {
    async generate(
        generator_settings: DynamicGeneratorSettings,
        on_progress: GenerateOnProgress,
    ): Promise<MainThreadWasmSudoku> {
        const RemoteWasmSudokuClass = await getRemoteWasmSudokuClass();
        const remoteWasmSudoku = await RemoteWasmSudokuClass.generate(generator_settings, on_progress);
        const serializedWasmSudoku = await remoteWasmSudoku.serialize();
        const wasmSudoku = WasmSudoku.deserialize(serializedWasmSudoku);
        return createProxy(wasmSudoku);
    },
} satisfies DelegatedConstructorFunctions;

function createProxy(inner: WasmSudoku): MainThreadWasmSudoku {
    const proxy = new Proxy(inner, {
        get(target, prop, receiver) {
            if (prop === "tryStrategies") {
                return remoteTryStrategies.bind(target);
            }
            return Reflect.get(target, prop, receiver) as unknown;
        },
    });
    return proxy as unknown as MainThreadWasmSudoku;
}

const remoteTryStrategies = async function (
    this: WasmSudoku,
    strategies: StrategySet,
): Promise<DynamicSolveStep | undefined> {
    return undefined;
} satisfies Asyncify<WasmSudoku["tryStrategies"]>;

function getRemoteWasmSudokuClass(): Promise<RemoteWasmSudokuClass> {
    throw new Error("Not implemented");
}
