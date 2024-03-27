import { atom, selector } from "recoil";
import type { IsEqual } from "type-fest";
import { z } from "zod";
import { assert } from "../../typeUtils";
import type { CellWorldDimensions, DynamicCells, TileIndex } from "../../types";
import { localStorageEffect } from "./localStorageEffect";
import { remoteWorkerApiState } from "./worker";

export type WorldView = z.infer<typeof worldViewSchema>;
export const worldViewSchema = z.enum(["sudoku", "map"]);

const usizeSchema = z
    .number()
    .nonnegative()
    .int()
    // wasm32 (bits)
    .max(Math.pow(2, 32) - 1);
export const tileIndexSchema = z.object({
    row: usizeSchema,
    column: usizeSchema,
});
assert<IsEqual<z.infer<typeof tileIndexSchema>, TileIndex>>();

export type GameMode = z.infer<typeof gameModeSchema>;
export const gameModeSchema = z.discriminatedUnion("mode", [
    z.object({
        mode: z.literal("sudoku"),
    }),
    z.object({
        mode: z.literal("world"),
        // TODO: another discriminated union
        //  currentTileIndex is only needed when view is "sudoku"
        view: worldViewSchema,
        currentTileIndex: tileIndexSchema,
    }),
]);

export const gameModeState = atom<GameMode>({
    key: "GameMode",
    default: {
        mode: "sudoku",
    },
    effects: [localStorageEffect(gameModeSchema)],
});

export const allWorldCellsState = atom<DynamicCells>({
    key: "AllWorldCells",
    default: selector({
        key: "DefaultAllWorldCells",
        get: async ({ get }) => {
            const { wasmCellWorldProxy } = get(remoteWorkerApiState);
            return await wasmCellWorldProxy.allWorldCells();
        },
    }),
});

export const cellWorldDimensionsState = atom<CellWorldDimensions>({
    key: "CellWorldDimensions",
    default: selector({
        key: "DefaultCellWorldDimensions",
        get: async ({ get }) => {
            const { wasmCellWorldProxy } = get(remoteWorkerApiState);
            return await wasmCellWorldProxy.dimensions();
        },
    }),
});
