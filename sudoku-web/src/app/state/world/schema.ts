import type { IsEqual } from "type-fest";
import * as z from "zod";
import { assert } from "../../../typeUtils";
import type { WorldDim, WorldPosition } from "../../../types";

const usizeSchema = z.int()
    .nonnegative()
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

export type WorldView = z.infer<typeof worldViewSchema>;
export const worldViewSchema = z.enum(["sudoku", "map"]);

export type GameModeWorld = z.infer<typeof gameModeWorldSchema>;
export const gameModeWorldSchema = z.object({
    mode: z.literal("world"),
    view: worldViewSchema,
    selectedGridPosition: worldGridPositionSchema,
});
