import ButtonBase from "@mui/material/ButtonBase";
import classNames from "classnames";
import * as _ from "lodash-es";
import { selector, selectorFamily, useRecoilValue } from "recoil";
import { useHandleValue } from "../actions/sudokuActions";
import { inputState } from "../state/input";
import { sudokuSideLengthState } from "../state/sudoku";
import { valueToString } from "../utils/sudoku";

const isSelectedState = selectorFamily<boolean, number>({
    key: "ValueButton.isSelected",
    get:
        (value) =>
        ({ get }) => {
            const input = get(inputState);
            return input.stickyMode && input.selectedValue === value;
        },
    cachePolicy_UNSTABLE: {
        eviction: "most-recent",
    },
});

type SelectorValueProps = {
    value: number;
};

const ValueButton: React.FunctionComponent<SelectorValueProps> = ({ value }: SelectorValueProps) => {
    const handleValue = useHandleValue();

    const isSelected = useRecoilValue(isSelectedState(value));

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
};

const selectorValuesState = selector({
    key: "Selector.values",
    get: ({ get }) => {
        const sideLength = get(sudokuSideLengthState);
        return _.range(1, sideLength + 1);
    },
});

export const ValueSelector = () => {
    const selectorValues = useRecoilValue(selectorValuesState);
    return (
        <div className="selector-container">
            <div className="selector">
                {selectorValues.map((value) => (
                    <ValueButton key={value} value={value} />
                ))}
            </div>
        </div>
    );
};
