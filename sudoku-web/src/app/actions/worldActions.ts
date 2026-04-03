import { useAtomCallback } from "jotai/utils";
import { useCallback } from "react";
import { gameState } from "../state/gameMode";
import { mainThreadWasmSudokuClassState, wasmSudokuState } from "../state/mainThread/wasmSudoku";
import {
    allWorldCellsInvalidateCounterState,
    assertGameModeWorld,
    remoteWasmCellWorldState,
    selectedGridPositionState,
} from "../state/world";
import { updateSudoku } from "./sudokuActions";

export function useShowWorldMap() {
    return useAtomCallback(
        useCallback(async (get, set) => {
            const wasmSudoku = await get(wasmSudokuState);
            const remoteWasmCellWorld = await get(remoteWasmCellWorldState);
            const selectedGridPosition = get(selectedGridPositionState);

            const dynamicGrid = wasmSudoku.toDynamicGrid();

            await remoteWasmCellWorld.setGridAt(dynamicGrid, selectedGridPosition);

            set(allWorldCellsInvalidateCounterState, (prev) => prev + 1);

            set(gameState, (prev) => {
                return {
                    ...assertGameModeWorld(prev),
                    view: "map" as const,
                };
            });
        }, []),
    );
}

export function usePlaySelectedGrid() {
    return useAtomCallback(
        useCallback(async (get, set) => {
            const remoteWasmCellWorld = await get(remoteWasmCellWorldState);
            const selectedGridPosition = get(selectedGridPositionState);
            const newGrid = await remoteWasmCellWorld.toGridAt(selectedGridPosition);

            const MainThreadWasmSudoku = await get(mainThreadWasmSudokuClassState);

            const newWasmSudoku = await MainThreadWasmSudoku.fromDynamicGrid(newGrid);
            set(wasmSudokuState, newWasmSudoku);

            updateSudoku({
                set,
                wasmSudoku: newWasmSudoku,
            });

            // Switch view to sudoku
            set(gameState, (prev) => {
                if (!(prev.mode === "world" && prev.view === "map")) {
                    console.warn("Unexpected game state", prev);
                    return prev;
                }
                return {
                    ...prev,
                    view: "sudoku" as const,
                };
            });
        }, []),
    );
}
