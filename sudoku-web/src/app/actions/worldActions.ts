import { useRecoilCallback } from "recoil";
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
    return useRecoilCallback(({ snapshot, set }) => async () => {
        const remoteWasmSudoku = await snapshot.getPromise(remoteWasmSudokuState);
        const remoteWasmCellWorld = await snapshot.getPromise(remoteWasmCellWorldState);
        const selectedGridPosition = await snapshot.getPromise(selectedGridPositionState);

        const dynamicGrid = await remoteWasmSudoku.toDynamicGrid();

        await remoteWasmCellWorld.setGridAt(dynamicGrid, selectedGridPosition);

        set(allWorldCellsInvalidateCounterState, (prev) => prev + 1);

        set(gameState, (prev) => {
            return {
                ...assertGameModeWorld(prev),
                view: "map" as const,
            };
        });
    });
}

export function usePlaySelectedGrid() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async () => {
                const remoteWasmCellWorld = await snapshot.getPromise(remoteWasmCellWorldState);
                const selectedGridPosition = await snapshot.getPromise(selectedGridPositionState);
                const newGrid = await remoteWasmCellWorld.toGridAt(selectedGridPosition);

                const RemoteWasmSudoku = await snapshot.getPromise(remoteWasmSudokuClassState);

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
            },
        [],
    );
}
