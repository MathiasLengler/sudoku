import { range } from "lodash-es";
import z from "zod";
import type { BaseEnum } from "../../types";
import { baseToSideLength } from "./sudoku";

export const BASE_MIN = 2;
export const BASE_MAX = 5;
export const BASE_MARKS = range(BASE_MIN, BASE_MAX + 1).map((base) => {
    return {
        value: base,
        label: baseToLabel(base),
    };
});
export function baseToLabel(base: number): string {
    const sideLength = baseToSideLength(base);

    return `${sideLength}x${sideLength}`;
}

// zod port of `BaseEnum`
export const baseSchema = z.int().min(BASE_MIN).max(BASE_MAX);

export function parseBase(base: number): BaseEnum {
    return baseSchema.parse(base) as BaseEnum;
}
