import type { Position } from "../../types";
import { atom, selector } from "recoil";

export interface BaseInput {
    stickyMode: boolean;
    candidateMode: boolean;
}

export interface NormalModeInput extends BaseInput {
    stickyMode: false;
    selectedPos: Position;
    // Used for restoring state on sticky mode toggle
    previouslySelectedValue: number;
}

export type CellAction = "set" | "delete";

export interface StickyChain {
    cellAction: CellAction;
    handledGridPositions: Position[];
}

export interface StickyModeInput extends BaseInput {
    stickyMode: true;
    selectedValue: number;
    // Is defined if the primary pointer is in the active buttons state and has interacted with at least one cell.
    // The first actively interacted cell defines the action type for all subsequent cells.
    stickyChain: StickyChain | undefined;
    // Used for restoring state on sticky mode toggle
    previouslySelectedPos: Position;
}

export type Input = NormalModeInput | StickyModeInput;

export const inputState = atom<Input>({
    key: "Input",
    default: {
        stickyMode: false,
        selectedPos: { column: 0, row: 0 },
        candidateMode: false,
        previouslySelectedValue: 1,
    },
});

// Defined in normal mode
export const selectedPosState = selector<Position | undefined>({
    key: "Input.selectedPos",
    get: ({ get }) => {
        const input = get(inputState);
        if (!input.stickyMode) {
            return input.selectedPos;
        }
    },
});

// Defined in sticky mode
const selectedValueState = selector<number | undefined>({
    key: "Input.selectedValue",
    get: ({ get }) => {
        const input = get(inputState);
        if (input.stickyMode) {
            return input.selectedValue;
        }
    },
});
