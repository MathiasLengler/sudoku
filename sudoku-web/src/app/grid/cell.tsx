import type * as React from "react";
import type * as CSS from "csstype";
import classnames from "classnames";
import { indexToPosition, valueToString } from "../utils";
import type { CellViewCandidates, CellViewValue, TransportCell } from "../../types";
import { inputState } from "../state/input";
import { sudokuBaseState } from "../state/sudoku";
import { useRecoilValue } from "recoil";
import { useHandlePosition } from "../sudokuActions";

function cellBackgroundClass(isSelected: boolean, isGuide: boolean) {
    if (isSelected) {
        return "cell--selected";
    }
    if (isGuide) {
        return "cell--guide";
    }
}

function cellColorClass(fixed: boolean, incorrectValue: boolean) {
    if (fixed) {
        return "cell--fixed";
    }
    if (incorrectValue) {
        return "cell--incorrect-value";
    } else {
        return "cell--user";
    }
}

interface CellValueProps {
    value: CellViewValue["value"];
}

const CellValue: React.FunctionComponent<CellValueProps> = props => {
    const { value } = props;
    return (
        <div className="cellValue">
            <span className="cellValueText">{valueToString(value)}</span>
        </div>
    );
};

interface CandidatesProps {
    candidates: CellViewCandidates["candidates"];
}

const Candidates = ({ candidates }: CandidatesProps) => {
    const base = useRecoilValue(sudokuBaseState);
    const input = useRecoilValue(inputState);

    return (
        <div className="candidates">
            {candidates.map(candidate => {
                // Candidates are 1 based, grid calculations are 0 based.
                const { column, row } = indexToPosition({ blockIndex: candidate - 1, base });

                const style: CSS.Properties = {
                    "--candidate-column": column,
                    "--candidate-row": row,
                };

                return (
                    <span
                        key={candidate}
                        className={classnames("candidate", {
                            "candidate--guide": input.stickyMode && input.selectedValue === candidate,
                        })}
                        style={style}
                    >
                        {valueToString(candidate)}
                    </span>
                );
            })}
        </div>
    );
};

interface CellProps {
    blockCellIndex: number;
    cell: TransportCell;
    isSelected: boolean;
    isGuide: boolean;
}

export const Cell = (props: CellProps) => {
    const { blockCellIndex, cell, isSelected, isGuide } = props;

    const { position: gridPosition } = cell;

    const base = useRecoilValue(sudokuBaseState);

    const cellPositionInBlock = indexToPosition({ blockIndex: blockCellIndex, base: base });

    const style: CSS.Properties = {
        "--cell-column": cellPositionInBlock.column,
        "--cell-row": cellPositionInBlock.row,
    };

    const cellClassNames = classnames(
        "cell",
        cellBackgroundClass(isSelected, isGuide),
        cellColorClass(cell.kind === "value" && cell.fixed, cell.incorrectValue)
    );

    const handlePosition = useHandlePosition();

    return (
        <div
            className={cellClassNames}
            style={style}
            onPointerDown={({ isPrimary, buttons, pointerId }) => {
                if (
                    // Left Mouse, Touch Contact, Pen contact
                    buttons !== 1 ||
                    !isPrimary
                ) {
                    return;
                }
                console.debug("onPointerDown", { isPrimary, buttons, pointerId });
                handlePosition(gridPosition).catch(console.error);
            }}
            onPointerEnter={({ isPrimary, buttons, pointerId }) => {
                if (
                    // Left Mouse, Touch Contact, Pen contact
                    buttons !== 1 ||
                    !isPrimary
                ) {
                    return;
                }
                console.debug("onPointerEnter", { isPrimary, buttons, pointerId });
                handlePosition(gridPosition).catch(console.error);
            }}
        >
            {cell.kind === "value" ? <CellValue value={cell.value} /> : <Candidates candidates={cell.candidates} />}
        </div>
    );
};
