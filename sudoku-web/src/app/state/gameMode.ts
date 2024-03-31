import { z } from "zod";
import { gameModeWorldSchema } from "./world";
import { localStorageEffect } from "./localStorageEffect";
import { atom, selector } from "recoil";

export type Game = z.infer<typeof gameSchema>;
export const gameSchema = z.discriminatedUnion("mode", [
    z.object({
        mode: z.literal("sudoku"),
    }),
    gameModeWorldSchema,
]);

export type GameMode = Game["mode"];

export const gameState = atom<Game>({
    key: "Game",
    default: {
        mode: "sudoku",
    },
    effects: [localStorageEffect(gameSchema)],
});

export const gameModeState = selector<GameMode>({
    key: "GameMode",
    get: ({ get }) => get(gameState).mode,
});
