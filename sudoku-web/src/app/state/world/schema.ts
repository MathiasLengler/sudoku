import type { IsEqual } from "type-fest";

import * as z from "zod";

import type { WorldCellDim, WorldCellPosition, WorldGridDim, WorldGridPosition, WorldObjectKey } from "../../../types";

const usizeSchema = z
    .int()
    .nonnegative()
    // wasm32 (bits)
    .max(Math.pow(2, 32) - 1);

type Unbranded<T> = Omit<T, WorldObjectKey>;

/**
 * Types `schema` as producing the pkg brand `T`.
 *
 * - The single cast point for world brands: the brand key has no runtime value.
 * - `schema`'s output must equal `T` without its brand.
 */
export function brandAs<T extends Record<WorldObjectKey, unknown>>() {
    return <S extends z.ZodType>(
        schema: IsEqual<z.output<S>, Unbranded<T>> extends true ? S : never,
    ): z.ZodType<T, z.input<S>> =>
        // oxlint-disable-next-line typescript/no-unsafe-type-assertion
        schema as unknown as z.ZodType<T, z.input<S>>;
}

export const worldPositionSchema = z.object({
    row: usizeSchema,
    column: usizeSchema,
});
export const worldCellPositionSchema = brandAs<WorldCellPosition>()(worldPositionSchema);
export const worldGridPositionSchema = brandAs<WorldGridPosition>()(worldPositionSchema);

export const worldDimSchema = z.object({
    rowCount: usizeSchema,
    columnCount: usizeSchema,
});
export const worldCellDimSchema = brandAs<WorldCellDim>()(worldDimSchema);
export const worldGridDimSchema = brandAs<WorldGridDim>()(worldDimSchema);

export type WorldView = z.infer<typeof worldViewSchema>;
export const worldViewSchema = z.enum(["sudoku", "map"]);

export type GameModeWorld = z.infer<typeof gameModeWorldSchema>;
export const gameModeWorldSchema = z.object({
    mode: z.literal("world"),
    view: worldViewSchema,
    selectedGridPosition: worldGridPositionSchema,
});
