import ButtonBase from "@mui/material/ButtonBase";
import classNames from "classnames";
import * as _ from "es-toolkit";
import { atom, useAtomValue, type Atom } from "jotai";
import { useHandleValue } from "../actions/sudokuActions";
import { inputState } from "../state/input";
import { sudokuSideLengthState } from "../state/sudoku";
import { valueToString } from "../utils/sudoku";
import { atomFamily } from "jotai/utils";

const isSelectedState = atomFamily<number, Atom<boolean>>((value) =>
    atom((get) => {
        const input = get(inputState);
        return input.stickyMode && input.selectedValue === value;
    }),
);

type SelectorValueProps = {
    value: number;
};

function ValueButton({ value }: SelectorValueProps) {
    const handleValue = useHandleValue();

    const isSelected = useAtomValue(isSelectedState(value));

    const buttonClassNames = classNames("selector-value", {
        "selector-value--selected": isSelected,
    });

    return (
        <ButtonBase
            className={buttonClassNames}
            sx={{ typography: "button" }}
            onClick={async () => {
                await handleValue(value);
            }}
        >
            <span className="selector-value-text">{valueToString(value)}</span>
        </ButtonBase>
    );
}

const selectorValuesState = atom(async (get) => {
    const sideLength = await get(sudokuSideLengthState);
    return _.range(1, sideLength + 1);
});

export function ValueSelector() {
    const selectorValues = useAtomValue(selectorValuesState);
    return (
        <div className="selector-container">
            <div className="selector">
                {selectorValues.map((value) => (
                    <ValueButton key={value} value={value} />
                ))}
            </div>
        </div>
    );
}
