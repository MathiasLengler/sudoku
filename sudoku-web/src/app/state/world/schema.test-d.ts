import type * as z from "zod";

import { expectTypeOf, test } from "vitest";

import type { WorldCellPosition, WorldGridDim, WorldGridPosition } from "../../../types";

import { brandAs, worldCellPositionSchema, worldDimSchema, worldPositionSchema } from "./schema";

test("brandAs rejects a schema of another shape", () => {
    // @ts-expect-error dim schema for a position brand
    brandAs<WorldCellPosition>()(worldDimSchema);
    brandAs<WorldGridDim>()(worldDimSchema);
});

test("brands don't mix", () => {
    expectTypeOf<WorldCellPosition>().not.toExtend<WorldGridPosition>();
    expectTypeOf<{ row: number; column: number }>().not.toExtend<WorldCellPosition>();
    expectTypeOf<z.output<typeof worldCellPositionSchema>>().toEqualTypeOf<WorldCellPosition>();
    expectTypeOf<z.output<typeof worldPositionSchema>>().not.toExtend<WorldCellPosition>();
});
