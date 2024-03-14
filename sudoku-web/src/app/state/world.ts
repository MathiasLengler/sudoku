import { z } from "zod";
import { localStorageEffect } from "./localStorageEffect";
import { atom } from "recoil";
import type { IsEqual } from "type-fest";
import type { TileIndex } from "../../types";
import { assert } from "../../typeUtils";

export type WorldView = z.infer<typeof worldViewSchema>;
export const worldViewSchema = z.enum(["single", "map"]);

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

export type GameModeWorldState = z.infer<typeof gameModeWorldStateSchema>;
export const gameModeWorldStateSchema = z.object({
    view: worldViewSchema,
    currentTileIndex: tileIndexSchema,
});

export type GameModeState = z.infer<typeof gameModeStateSchema>;
export const gameModeStateSchema = z.discriminatedUnion("mode", [
    z.object({
        mode: z.literal("single"),
    }),
    z.object({
        mode: z.literal("world"),
        state: gameModeWorldStateSchema,
    }),
]);

export const cellWorldGameState = atom<GameModeState>({
    key: "GameModeState",
    default: {
        mode: "single",
    },
    effects: [localStorageEffect(gameModeStateSchema)],
});
