import type { DynamicPosition } from "../../types";
import { atom, selector } from "recoil";
import { localStorageEffect } from "./localStorageEffect";
import { z } from "zod";

const baseInputSchema = z.object({
    // Sticky mode:
    //  Select value, then cell position
    // Normal mode:
    //  Select cell position, then value
    stickyMode: z.boolean(),
    candidateMode: z.boolean(),
});
export type BaseInput = z.infer<typeof baseInputSchema>;

const positionSchema = z.object({ row: z.number().int().nonnegative(), column: z.number().int().nonnegative() });
const valueSchema = z.number().int().positive();

const normalModeInputSchema = z.object({
    stickyMode: z.literal(false),
    selectedPos: positionSchema,
    // Used for restoring state on sticky mode toggle
    previouslySelectedValue: valueSchema,
});
export type NormalModeInput = z.infer<typeof normalModeInputSchema>;

const cellActionSchema = z.enum(["set", "delete"]);
export type CellAction = z.infer<typeof cellActionSchema>;

export type StickyChain = z.infer<typeof stickyChainSchema>;
const stickyChainSchema = z.object({
    cellAction: cellActionSchema,
    handledGridPositions: positionSchema.array(),
});
export type StickyModeInput = z.infer<typeof stickyModeInputSchema>;
const stickyModeInputSchema = z.object({
    stickyMode: z.literal(true),
    selectedValue: valueSchema,
    // Is defined if the primary pointer is in the active buttons state and has interacted with at least one cell.
    // The first actively interacted cell defines the action type for all subsequent cells.
    stickyChain: stickyChainSchema.optional(),
    // Used for restoring state on sticky mode toggle
    previouslySelectedPos: positionSchema,
});

export type Input = z.infer<typeof inputSchema>;
const inputSchema = z
    .discriminatedUnion("stickyMode", [normalModeInputSchema, stickyModeInputSchema])
    .and(baseInputSchema);

export const inputState = atom<Input>({
    key: "Input",
    default: {
        stickyMode: false,
        selectedPos: { column: 0, row: 0 },
        candidateMode: false,
        previouslySelectedValue: 1,
    },
    effects: [localStorageEffect(inputSchema)],
});

// Defined in normal mode
export const selectedPosState = selector<DynamicPosition | undefined>({
    key: "Input.selectedPos",
    get: ({ get }) => {
        const input = get(inputState);
        if (!input.stickyMode) {
            return input.selectedPos;
        }
    },
});

export const inputCandidateModeState = selector<Input["candidateMode"]>({
    key: "Input.candidateMode",
    get: ({ get }) => get(inputState).candidateMode,
});

export const inputStickyModeState = selector<Input["stickyMode"]>({
    key: "Input.stickyMode",
    get: ({ get }) => get(inputState).stickyMode,
});
