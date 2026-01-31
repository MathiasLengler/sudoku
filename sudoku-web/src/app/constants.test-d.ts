import { expectTypeOf, test } from "vitest";
import type * as z from "zod";
import type { GoalOptimization, GridFormatEnum, GridMetric, GridMetricName, StrategyEnum } from "../types";
import {
    strategyEnumSchema,
    type goalOptimizationSchema,
    type gridFormatSchema,
    type gridMetricNameSchema,
    type gridMetricSchema,
} from "./constants";

test("sudoku-rs bindings match zod schema types", () => {
    expectTypeOf<z.output<typeof strategyEnumSchema>>().toEqualTypeOf<StrategyEnum>();

    expectTypeOf<z.output<typeof gridFormatSchema>>().toEqualTypeOf<GridFormatEnum>();

    expectTypeOf<z.output<typeof gridMetricNameSchema>>().toEqualTypeOf<GridMetricName>();

    // `toEqualTypeOf` is too strict here because the discriminated unions are defined slightly differently.
    expectTypeOf<z.output<typeof gridMetricSchema>>().toExtend<GridMetric>();
    expectTypeOf<GridMetric>().toExtend<z.output<typeof gridMetricSchema>>();

    expectTypeOf<z.output<typeof goalOptimizationSchema>>().toEqualTypeOf<GoalOptimization>();

    // TODO: assert:
    //  strategyMapKeySchema
    //  strategySetSchema
});
