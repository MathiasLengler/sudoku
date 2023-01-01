import { SelectorCallbackInterface, useRecoilCallback } from "recoil";
import { sudokuSideLengthState, sudokuState, wasmSudokuProxyContainerState } from "./state/sudoku";
import type { DynamicGeneratorSettings, DynamicStrategy, Position } from "../../../sudoku-rs/bindings";
import { Input, inputState } from "./state/input";
import { cellAtGridPositionState } from "./state/cellIndexing";
import _ from "lodash";
import type { GridFormat } from "../types";
import type { WasmSudokuProxy } from "../spawnWorker";

// Snapshot accessors
async function getWasmSudokuProxy({ snapshot }: Pick<SelectorCallbackInterface, "snapshot">): Promise<WasmSudokuProxy> {
    const { wasmSudokuProxy } = await snapshot.getPromise(wasmSudokuProxyContainerState);
    console.log({ wasmSudokuProxy });
    return wasmSudokuProxy;
}

async function getInput({ snapshot }: Pick<SelectorCallbackInterface, "snapshot">) {
    return await snapshot.getPromise(inputState);
}

// Validation
async function isFixedValueCell({
    snapshot,
    gridPosition,
}: Pick<SelectorCallbackInterface, "snapshot"> & { gridPosition: Position }) {
    const cell = await snapshot.getPromise(cellAtGridPositionState(gridPosition));

    if (cell.kind === "value" && cell.fixed) {
        console.warn("Can't modify fixed cell", cell);

        return true;
    } else {
        return false;
    }
}

async function isInvalidValue({ snapshot, value }: Pick<SelectorCallbackInterface, "snapshot"> & { value: number }) {
    const sideLength = await snapshot.getPromise(sudokuSideLengthState);

    if (!_.inRange(value, 0, sideLength + 1)) {
        console.warn(`Skip handling of value ${value} outside range [0, ${sideLength}]`);
        return true;
    } else {
        return false;
    }
}

async function isInvalidGridPosition({
    snapshot,
    gridPosition,
}: Pick<SelectorCallbackInterface, "snapshot"> & { gridPosition: Position }) {
    const sideLength = await snapshot.getPromise(sudokuSideLengthState);

    if (!_.inRange(gridPosition.row, 0, sideLength) || !_.inRange(gridPosition.column, 0, sideLength)) {
        console.warn(
            `Skip handling of grid position ${JSON.stringify(
                gridPosition
            )} with coordinate outside range [0, ${sideLength})`
        );
        return true;
    } else {
        return false;
    }
}

// Mutation helpers
async function updateSudoku({
    set,
    wasmSudokuProxy,
}: Pick<SelectorCallbackInterface, "set"> & { wasmSudokuProxy: WasmSudokuProxy }) {
    const newSudoku = await wasmSudokuProxy.getSudoku();
    set(sudokuState, newSudoku);
}

async function applyValueAtGridPosition({
    snapshot,
    set,
    value,
    gridPosition,
}: Pick<SelectorCallbackInterface, "snapshot" | "set"> & {
    value: number;
    gridPosition: Position;
}) {
    if (await isFixedValueCell({ snapshot, gridPosition })) {
        return;
    }
    const input = await getInput({ snapshot });

    const wasmSudokuProxy = await getWasmSudokuProxy({ snapshot });

    if (value === 0) {
        await wasmSudokuProxy.delete(gridPosition);
    } else {
        if (input.candidateMode) {
            await wasmSudokuProxy.toggleCandidate(gridPosition, value);
        } else {
            await wasmSudokuProxy.setOrToggleValue(gridPosition, value);
        }
    }
    await updateSudoku({ set, wasmSudokuProxy });
}

// Public action hooks
export function useHandlePosition() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async (gridPosition: Position) => {
                if (await isInvalidGridPosition({ snapshot, gridPosition })) {
                    return;
                }
                const input = await getInput({ snapshot });
                if (input.stickyMode) {
                    await applyValueAtGridPosition({ set, snapshot, gridPosition, value: input.selectedValue });
                } else {
                    set(inputState, input => ({ ...input, selectedPos: gridPosition }));
                }
            },
        []
    );
}

export function useHandleValue() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async (value: number) => {
                if (await isInvalidValue({ snapshot, value })) {
                    return;
                }

                const input = await getInput({ snapshot });
                if (input.stickyMode) {
                    set(inputState, input => ({ ...input, selectedValue: value }));
                } else {
                    await applyValueAtGridPosition({ set, snapshot, gridPosition: input.selectedPos, value });
                }
            },
        []
    );
}
export function useDeleteSelectedCell() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async () => {
                const input = await getInput({ snapshot });
                if (input.stickyMode) {
                    console.warn("Deletion of cells is unavailable in sticky mode");
                } else {
                    await applyValueAtGridPosition({
                        set,
                        snapshot,
                        gridPosition: input.selectedPos,
                        value: 0,
                    });
                }
            },
        []
    );
}

export function useSetAllDirectCandidates() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async () => {
                const wasmSudokuProxy = await getWasmSudokuProxy({ snapshot });
                await wasmSudokuProxy.setAllDirectCandidates();
                await updateSudoku({ set, wasmSudokuProxy });
            },
        []
    );
}
export function useUndo() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async () => {
                const wasmSudokuProxy = await getWasmSudokuProxy({ snapshot });
                await wasmSudokuProxy.undo();
                await updateSudoku({ set, wasmSudokuProxy });
            },
        []
    );
}
export function useRedo() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async () => {
                const wasmSudokuProxy = await getWasmSudokuProxy({ snapshot });
                await wasmSudokuProxy.redo();
                await updateSudoku({ set, wasmSudokuProxy });
            },
        []
    );
}

export function useGenerate() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async (settings: DynamicGeneratorSettings) => {
                const wasmSudokuProxy = await getWasmSudokuProxy({ snapshot });
                await wasmSudokuProxy.generate(settings);
                await updateSudoku({ set, wasmSudokuProxy });
            },
        []
    );
}
export function useImportSudokuString() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async (input: string) => {
                const wasmSudokuProxy = await getWasmSudokuProxy({ snapshot });
                await wasmSudokuProxy.import(input);
                await updateSudoku({ set, wasmSudokuProxy });
            },
        []
    );
}
export function useExportSudokuString() {
    return useRecoilCallback(
        ({ snapshot }) =>
            async (format: GridFormat) => {
                const wasmSudokuProxy = await getWasmSudokuProxy({ snapshot });
                return await wasmSudokuProxy.export(format);
            },
        []
    );
}

export function useTryStrategy() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async (strategyName: DynamicStrategy) => {
                const wasmSudokuProxy = await getWasmSudokuProxy({ snapshot });
                const res = await wasmSudokuProxy.tryStrategy(strategyName);
                await updateSudoku({ set, wasmSudokuProxy });
                return res;
            },
        []
    );
}
export function useToggleCandidateMode() {
    return useRecoilCallback(
        ({ set }) =>
            () => {
                set(inputState, input => ({ ...input, candidateMode: !input.candidateMode }));
            },
        []
    );
}
export function useToggleStickyMode() {
    return useRecoilCallback(
        ({ set }) =>
            () => {
                set(inputState, (input): Input => {
                    if (input.stickyMode) {
                        return {
                            stickyMode: false,
                            candidateMode: input.candidateMode,
                            selectedPos: input.inactiveSelectedPos,
                            inactiveSelectedValue: input.selectedValue,
                        };
                    } else {
                        return {
                            stickyMode: true,
                            candidateMode: input.candidateMode,
                            selectedValue: input.inactiveSelectedValue,
                            inactiveSelectedPos: input.selectedPos,
                        };
                    }
                });
            },
        []
    );
}
