import type { Position } from "../../types";
import { atom, selector } from "recoil";

interface BaseInput {
    stickyMode: boolean;
    candidateMode: boolean;
}

interface NormalModeInput extends BaseInput {
    stickyMode: false;
    selectedPos: Position;
    // Used for restoring state on sticky mode toggle
    inactiveSelectedValue: number;
}
interface StickyModeInput extends BaseInput {
    stickyMode: true;
    selectedValue: number;
    // Used for restoring state on sticky mode toggle
    inactiveSelectedPos: Position;
}

export type Input = NormalModeInput | StickyModeInput;

export const inputState = atom<Input>({
    key: "Input",
    default: {
        stickyMode: false,
        selectedPos: { column: 0, row: 0 },
        candidateMode: false,
        inactiveSelectedValue: 1,
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
