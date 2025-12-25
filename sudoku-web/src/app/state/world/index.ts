import { atom, type Atom } from "jotai";
import { atomFamily, RESET } from "jotai/utils";
import * as _ from "es-toolkit";
import { WasmCellWorld } from "sudoku-wasm";
import type { CellWorldDimensions, DynamicCell, DynamicCells, DynamicPosition } from "../../../types";
import { init } from "../../state/worker/bg/init";
import { validateCellWorldPosition } from "../../utils/world";
import { gameState, type Game } from "../gameMode";
import { sudokuBaseState, sudokuSideLengthState } from "../sudoku";
import { remoteWasmCellWorldClassState, type RemoteWasmCellWorld } from "../worker";
import { fixupComlinkRemote } from "../worker/comlinkProxyWrapper";
import {
    worldGridDimSchema,
    worldGridPositionSchema,
    type GameModeWorld,
    type WorldCellDim,
    type WorldCellPosition,
    type WorldGridDim,
    type WorldGridPosition,
} from "./schema";

export const DEFAULT_WORLD_GRID_POSITION = worldGridPositionSchema.parse({ row: 0, column: 0 });

export const showWorldMapState = atom<boolean>((get) => {
    const game = get(gameState);
    return game.mode === "world" && game.view === "map";
});

export const requestedGridDimState = atom<WorldGridDim>(worldGridDimSchema.parse({ rowCount: 3, columnCount: 3 }));

export const requestedOverlapState = atom<number>(1);
export const requestedSeedState = atom<bigint>(1n);

export const remoteWasmCellWorldState = atom<Promise<RemoteWasmCellWorld>>(async (get) => {
    const RemoteWasmCellWorldClass = await get(remoteWasmCellWorldClassState);
    const requestedWorldBase = await get(sudokuBaseState);
    const requestedGridDim = get(requestedGridDimState);
    const requestedOverlap = get(requestedOverlapState);
    const requestedSeed = get(requestedSeedState);

    return fixupComlinkRemote(
        await RemoteWasmCellWorldClass.generate(requestedWorldBase, requestedGridDim, requestedOverlap, requestedSeed),
    );
});

export const emptyWasmCellWorldState = atom<Promise<WasmCellWorld>>(async (get) => {
    await init(1);

    const requestedWorldBase = await get(sudokuBaseState);
    const requestedGridDim = get(requestedGridDimState);
    const requestedOverlap = get(requestedOverlapState);

    return WasmCellWorld.new(requestedWorldBase, requestedGridDim, requestedOverlap);
});

export const allWorldCellsInvalidateCounterState = atom<number>(0);

export const allWorldCellsState = atom<Promise<DynamicCells>>(async (get) => {
    get(allWorldCellsInvalidateCounterState);
    const remoteWasmCellWorld = await get(remoteWasmCellWorldState);
    return await remoteWasmCellWorld.allWorldCells();
});

export const worldCellSizeState = atom<number>(100);

export const cellWorldDimensionsState = atom<Promise<CellWorldDimensions>>(async (get) => {
    const remoteWasmCellWorld = await get(remoteWasmCellWorldState);
    return await remoteWasmCellWorld.dimensions();
});

export const gridDimState = atom<Promise<WorldGridDim>>(async (get) => (await get(cellWorldDimensionsState)).gridDim);
export const cellDimState = atom<Promise<WorldCellDim>>(async (get) => (await get(cellWorldDimensionsState)).cellDim);
export const overlapState = atom<Promise<number>>(async (get) => (await get(cellWorldDimensionsState)).overlap);

export const gridStrideState = atom<Promise<number>>(
    async (get) => (await get(sudokuSideLengthState)) - (await get(overlapState)),
);

export function assertGameModeWorld(gameMode: Game): GameModeWorld {
    if (gameMode.mode !== "world") {
        throw new Error(`Expected game mode 'world', instead got: ${gameMode.mode}`);
    }
    return gameMode;
}

export const selectedGridPositionState = atom<WorldGridPosition, [WorldGridPosition | typeof RESET], void>(
    (get) => {
        const gameModeWorld = assertGameModeWorld(get(gameState));
        return gameModeWorld.selectedGridPosition;
    },
    (get, set, newGridIndex) => {
        const gameModeWorld = assertGameModeWorld(get(gameState));

        set(gameState, {
            ...gameModeWorld,
            selectedGridPosition: newGridIndex === RESET ? DEFAULT_WORLD_GRID_POSITION : newGridIndex,
        });
    },
);

export const selectedGridRowIndexState = atom<number>((get) => get(selectedGridPositionState).row);
export const selectedGridColumnIndexState = atom<number>((get) => get(selectedGridPositionState).column);
export const selectedGridBaseCellRowIndexState = atom<Promise<number>>(
    async (get) => get(selectedGridRowIndexState) * (await get(gridStrideState)),
);
export const selectedGridBaseCellColumnIndexState = atom<Promise<number>>(
    async (get) => get(selectedGridColumnIndexState) * (await get(gridStrideState)),
);

export const isCellInSelectedGridState = atomFamily<DynamicPosition, Atom<Promise<boolean>>>(
    (cellWorldPosition) =>
        atom(async (get) => {
            const { row: cellRowIndex, column: cellColumnIndex } = cellWorldPosition;
            const gridSideLength = await get(sudokuSideLengthState);
            const selectedGridBaseCellRowIndex = await get(selectedGridBaseCellRowIndexState);
            const selectedGridBaseCellColumnIndex = await get(selectedGridBaseCellColumnIndexState);

            const isCellInSelectedGrid =
                _.inRange(cellRowIndex, selectedGridBaseCellRowIndex, selectedGridBaseCellRowIndex + gridSideLength) &&
                _.inRange(
                    cellColumnIndex,
                    selectedGridBaseCellColumnIndex,
                    selectedGridBaseCellColumnIndex + gridSideLength,
                );
            return isCellInSelectedGrid;
        }),
    _.isEqual,
);

export const worldCellState = atomFamily<WorldCellPosition, Atom<Promise<DynamicCell>>>((cellWorldPosition) =>
    atom(async (get) => {
        const cellDim = await get(cellDimState);

        validateCellWorldPosition({ cellWorldPosition, cellDim });

        const allWorldCells = await get(allWorldCellsState);
        return allWorldCells[cellWorldPosition.row * cellDim.columnCount + cellWorldPosition.column]!;
    }),
);
