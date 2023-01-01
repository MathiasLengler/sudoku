import type * as React from "react";
import type { PointerEventHandler } from "react";
import type * as CSS from "csstype";
import classnames from "classnames";
import { indexToPosition, positionToIndex, valueToString } from "../utils";
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

    const gridIndex = positionToIndex({ gridPosition, base });
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

    // TODO: replace with onPointerEnter
    const onPointerMove: PointerEventHandler = e => {
        // Left Mouse, Touch Contact, Pen contact
        if (e.buttons !== 1) {
            return;
        }

        handlePosition(gridPosition).catch(console.error);

        // Workaround for touch drag cell selection
        if (e.pointerType !== "mouse") {
            let el = document.elementFromPoint(e.clientX, e.clientY);
            if (el) {
                while (el.parentElement !== null) {
                    if (el.classList.contains("cell")) {
                        el.setPointerCapture(e.pointerId);
                        break;
                    }
                    el = el.parentElement;
                }
            }
        }
    };

    return (
        <div
            className={cellClassNames}
            style={style}
            onPointerDown={() => {
                console.log("onPointerDown");
                handlePosition(gridPosition).catch(console.error);
            }}
            onPointerEnter={e => {
                console.log("onPointerEnter", gridPosition, gridIndex);
            }}
            onPointerMove={onPointerMove}
        >
            {cell.kind === "value" ? <CellValue value={cell.value} /> : <Candidates candidates={cell.candidates} />}
        </div>
    );
};
