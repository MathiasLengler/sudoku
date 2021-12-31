import * as React from "react";
import { PointerEventHandler } from "react";
import * as CSS from "csstype";
import isEqual from "lodash/isEqual";
import classnames from "classnames";
import { indexToPosition, valueToString } from "../utils";
import { Input, WasmSudokuController } from "../wasmSudokuController";

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
    value: ValueCell["value"];
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
    candidates: CandidatesCell["candidates"];
    base: TransportSudoku["base"];
    selectedValue: Input["selectedValue"];
    stickyMode: Input["stickyMode"];
}

const Candidates: React.FunctionComponent<CandidatesProps> = ({ base, candidates, selectedValue, stickyMode }) => {
    return (
        <div className="candidates">
            {candidates.map(candidate => {
                // Candidates are 1 based, grid calculations are 0 based.
                const { column, row } = indexToPosition(candidate - 1, base);

                const style: CSS.Properties = {
                    "--candidate-column": column,
                    "--candidate-row": row,
                };

                return (
                    <span
                        key={candidate}
                        className={classnames("candidate", {
                            "candidate--guide": stickyMode && selectedValue === candidate,
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

export const MemoCandidates = React.memo(Candidates, isEqual);

interface CellProps {
    blockCellIndex: number;
    cell: TransportCell;
    base: TransportSudoku["base"];
    isSelected: boolean;
    isGuide: boolean;
    sudokuController: WasmSudokuController;
    selectedValue: Input["selectedValue"];
    stickyMode: Input["stickyMode"];
}

const Cell: React.FunctionComponent<CellProps> = props => {
    const { blockCellIndex, cell, base, isSelected, isGuide, sudokuController, selectedValue, stickyMode } = props;

    const { position: gridPosition } = cell;

    const blockCellPosition = indexToPosition(blockCellIndex, base);

    const style: CSS.Properties = {
        "--cell-column": blockCellPosition.column,
        "--cell-row": blockCellPosition.row,
    };

    const cellClassNames = classnames(
        "cell",
        cellBackgroundClass(isSelected, isGuide),
        cellColorClass(cell.fixed, cell.incorrectValue)
    );

    const onPointerMove: PointerEventHandler = e => {
        // Left Mouse, Touch Contact, Pen contact
        if (e.buttons !== 1) {
            return;
        }

        sudokuController.handlePosition(gridPosition, true);

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
            onPointerDown={() => sudokuController.handlePosition(gridPosition)}
            onPointerMove={onPointerMove}
        >
            {cell.kind === "value" ? (
                <CellValue value={cell.value} />
            ) : (
                <MemoCandidates
                    candidates={cell.candidates}
                    base={base}
                    selectedValue={selectedValue}
                    stickyMode={stickyMode}
                />
            )}
        </div>
    );
};
export const MemoCell = React.memo(Cell, isEqual);
