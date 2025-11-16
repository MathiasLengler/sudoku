import { useAtomCallback } from "jotai/utils";
import { useCallback } from "react";
import { gameState } from "../state/gameMode";
import { remoteWasmSudokuClassState, remoteWasmSudokuState } from "../state/worker";
import { fixupComlinkProxy } from "../state/worker/comlinkProxyWrapper";
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
            const remoteWasmSudoku = await get(remoteWasmSudokuState);
            const remoteWasmCellWorld = await get(remoteWasmCellWorldState);
            const selectedGridPosition = get(selectedGridPositionState);

            const dynamicGrid = await remoteWasmSudoku.toDynamicGrid();

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

            const RemoteWasmSudoku = await get(remoteWasmSudokuClassState);

            const newRemoteWasmSudoku = fixupComlinkProxy(await RemoteWasmSudoku.from_dynamic_grid(newGrid));
            set(remoteWasmSudokuState, newRemoteWasmSudoku);

            await updateSudoku({
                set,
                wasmSudokuProxy: newRemoteWasmSudoku,
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
