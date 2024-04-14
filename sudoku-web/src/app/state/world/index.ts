import { DefaultValue, atom, selector, selectorFamily } from "recoil";
import type { IsEqual } from "type-fest";
import { z } from "zod";
import { assert, type CreateSerializableParam } from "../../../typeUtils";
import type {
    BaseEnum,
    CellWorldDimensions,
    DynamicCell,
    DynamicCells,
    DynamicPosition,
    WorldDim,
    WorldPosition,
} from "../../../types";
import { type Game, gameState } from "../gameMode";
import { remoteWasmCellWorldClassState, type RemoteWasmCellWorld } from "../worker";
import { sudokuBaseState, sudokuSideLengthState } from "../sudoku";
import _ from "lodash";
import { validateCellWorldPosition } from "../../utils/world";
import { fixupComlinkProxy } from "../worker/comlinkProxyWrapper";
import { WasmCellWorld } from "../../../../../sudoku-wasm/pkg";
import { init } from "../../state/worker/bg/init";

export type WorldView = z.infer<typeof worldViewSchema>;
export const worldViewSchema = z.enum(["sudoku", "map"]);

const usizeSchema = z
    .number()
    .nonnegative()
    .int()
    // wasm32 (bits)
    .max(Math.pow(2, 32) - 1);
export const worldPositionSchema = z.object({
    row: usizeSchema,
    column: usizeSchema,
});
assert<IsEqual<z.infer<typeof worldPositionSchema>, WorldPosition>>();

export type WorldCellPosition = z.infer<typeof worldCellPositionSchema>;
export const worldCellPositionSchema = worldPositionSchema.brand("WorldCellPosition");

export type WorldGridPosition = z.infer<typeof worldGridPositionSchema>;
export const worldGridPositionSchema = worldPositionSchema.brand("WorldGridPosition");

export const worldDimSchema = z.object({
    rowCount: usizeSchema,
    columnCount: usizeSchema,
});
assert<IsEqual<z.infer<typeof worldDimSchema>, WorldDim>>();

export type WorldCellDim = z.infer<typeof worldCellDimSchema>;
export const worldCellDimSchema = worldDimSchema.brand("WorldCellDim");

export type WorldGridDim = z.infer<typeof worldGridDimSchema>;
export const worldGridDimSchema = worldDimSchema.brand("WorldGridDim");

export type GameModeWorld = z.infer<typeof gameModeWorldSchema>;
export const gameModeWorldSchema = z.object({
    mode: z.literal("world"),
    view: worldViewSchema,
    selectedGridIndex: worldPositionSchema,
});

export const showWorldMapState = selector<boolean>({
    key: "showWorldMap",
    get: ({ get }) => {
        const game = get(gameState);
        return game.mode === "world" && game.view === "map";
    },
});

export const requestedGridDimState = atom<WorldGridDim>({
    key: "requestedGridDimState",
    default: worldGridDimSchema.parse({ rowCount: 3, columnCount: 3 }),
});

export const requestedOverlapState = atom<number>({
    key: "requestedOverlapState",
    default: 1,
});
export const requestedSeedState = atom<bigint>({
    key: "requestedSeedState",
    default: 1n,
});

export const remoteWasmCellWorldState = selector<RemoteWasmCellWorld>({
    key: "remoteWasmCellWorldState",
    get: async ({ get }) => {
        const RemoteWasmCellWorldClass = get(remoteWasmCellWorldClassState);
        const requestedWorldBase = get(sudokuBaseState);
        const requestedGridDim = get(requestedGridDimState);
        const requestedOverlap = get(requestedOverlapState);
        const requestedSeed = get(requestedSeedState);

        return fixupComlinkProxy(
            await RemoteWasmCellWorldClass.generate(
                requestedWorldBase,
                requestedGridDim,
                requestedOverlap,
                requestedSeed,
            ),
        );
    },
});

export const emptyWasmCellWorldState = selector<WasmCellWorld>({
    key: "emptyWasmCellWorldState",
    get: async ({ get }) => {
        await init(1);

        const requestedWorldBase = get(sudokuBaseState);
        const requestedGridDim = get(requestedGridDimState);
        const requestedOverlap = get(requestedOverlapState);

        return new WasmCellWorld(requestedWorldBase, requestedGridDim, requestedOverlap);
    },
});

export const allWorldCellsState = selector<DynamicCells>({
    key: "AllWorldCells",
    get: async ({ get }) => {
        const remoteWasmCellWorld = get(remoteWasmCellWorldState);
        return await remoteWasmCellWorld.allWorldCells();
    },
});

export const worldCellSizeState = atom<number>({
    key: "WorldCellSize",
    default: 100,
});

export const cellWorldDimensionsState = selector<CellWorldDimensions>({
    key: "CellWorldDimensions",
    get: async ({ get }) => {
        const remoteWasmCellWorld = get(remoteWasmCellWorldState);
        return await remoteWasmCellWorld.dimensions();
    },
});

export const gridDimState = selector<WorldGridDim>({
    key: "gridDim",
    get: ({ get }) => get(cellWorldDimensionsState).gridDim,
});
export const cellDimState = selector<WorldCellDim>({
    key: "cellDim",
    get: ({ get }) => get(cellWorldDimensionsState).cellDim,
});
export const overlapState = selector<number>({
    key: "overlap",
    get: ({ get }) => get(cellWorldDimensionsState).overlap,
});

export const gridStrideState = selector<number>({
    key: "gridStride",
    get: ({ get }) => get(sudokuSideLengthState) - get(overlapState),
});

function assertGameModeWorld(gameMode: Game): GameModeWorld {
    if (gameMode.mode !== "world") {
        throw new Error(`Expected game mode 'world', instead got: ${gameMode.mode}`);
    }
    return gameMode;
}

export const selectedGridIndexState = selector<WorldPosition>({
    key: "SelectedGridIndex",
    get: ({ get }) => {
        const gameModeWorld = assertGameModeWorld(get(gameState));
        return gameModeWorld.selectedGridIndex;
    },
    set: ({ set }, newGridIndex) => {
        set(gameState, (prevGameMode) => {
            const gameModeWorld = assertGameModeWorld(prevGameMode);

            return {
                ...gameModeWorld,
                selectedGridIndex: newGridIndex instanceof DefaultValue ? { row: 0, column: 0 } : newGridIndex,
            };
        });
    },
});

export const selectedGridRowIndexState = selector<number>({
    key: "selectedGridRowIndex",
    get: ({ get }) => get(selectedGridIndexState).row,
});
export const selectedGridColumnIndexState = selector<number>({
    key: "selectedGridColumnIndex",
    get: ({ get }) => get(selectedGridIndexState).column,
});
export const selectedGridBaseCellRowIndexState = selector<number>({
    key: "selectedGridBaseCellRowIndex",
    get: ({ get }) => get(selectedGridRowIndexState) * get(gridStrideState),
});
export const selectedGridBaseCellColumnIndexState = selector<number>({
    key: "selectedGridBaseCellColumnIndex",
    get: ({ get }) => get(selectedGridColumnIndexState) * get(gridStrideState),
});

export const isCellInSelectedGridState = selectorFamily<boolean, CreateSerializableParam<DynamicPosition>>({
    key: "isCellInSelectedGrid",
    get:
        (cellWorldPosition) =>
        ({ get }) => {
            const { row: cellRowIndex, column: cellColumnIndex } = cellWorldPosition;
            const gridSideLength = get(sudokuSideLengthState);
            const selectedGridBaseCellRowIndex = get(selectedGridBaseCellRowIndexState);
            const selectedGridBaseCellColumnIndex = get(selectedGridBaseCellColumnIndexState);

            const isCellInSelectedGrid =
                _.inRange(cellRowIndex, selectedGridBaseCellRowIndex, selectedGridBaseCellRowIndex + gridSideLength) &&
                _.inRange(
                    cellColumnIndex,
                    selectedGridBaseCellColumnIndex,
                    selectedGridBaseCellColumnIndex + gridSideLength,
                );
            return isCellInSelectedGrid;
        },
    cachePolicy_UNSTABLE: {
        eviction: "most-recent",
    },
});

export const worldCellState = selectorFamily<DynamicCell, WorldCellPosition>({
    key: "worldCell",
    get:
        (cellWorldPosition) =>
        ({ get }) => {
            const cellDim = get(cellDimState);

            validateCellWorldPosition({ cellWorldPosition, cellDim });

            const allWorldCells = get(allWorldCellsState);
            return allWorldCells[cellWorldPosition.row * cellDim.columnCount + cellWorldPosition.column]!;
        },
});
