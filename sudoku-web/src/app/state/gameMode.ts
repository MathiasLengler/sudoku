import { atom } from "jotai";
import { atomWithStorage } from "jotai/utils";
import * as z from "zod";
import { getZodLocalStorage } from "./localStorageEffect";
import { gameModeWorldSchema } from "./world/schema";

export type Game = z.infer<typeof gameSchema>;
export const gameSchema = z.discriminatedUnion("mode", [
    z.object({
        mode: z.literal("sudoku"),
    }),
    gameModeWorldSchema,
]);

export type GameMode = Game["mode"];

export const gameState = atomWithStorage<Game>("gameState", { mode: "sudoku" }, getZodLocalStorage(gameSchema));

export const gameModeState = atom<GameMode>((get) => get(gameState).mode);
