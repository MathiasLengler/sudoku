import type * as React from "react";
import type * as CSS from "csstype";
import classNames from "classnames";
import { cellColorClass, indexToPosition, valueToString } from "../utils/sudoku";
import type { DynamicCellCandidates, DynamicCellValue, DynamicPosition, TransportCell } from "../../types";
import { inputState } from "../state/input";
import { sudokuBaseState } from "../state/sudoku";
import { useRecoilValue } from "recoil";
import { useHandlePosition } from "../actions/sudokuActions";
import { hintState } from "../state/hint";

import isEqual from "lodash/isEqual";

function cellBackgroundClass(isSelected: boolean, isGuide: boolean) {
    if (isSelected) {
        return "cell--selected";
    }
    if (isGuide) {
        return "cell--guide";
    }
}

type CellValueProps = {
    value: DynamicCellValue["value"];
};

export const CellValue: React.FunctionComponent<CellValueProps> = (props) => {
    const { value } = props;
    return <div className="cell-value">{valueToString(value)}</div>;
};

type CandidatesProps = {
    candidates: DynamicCellCandidates["candidates"];
    gridPosition: DynamicPosition;
    showGuide?: boolean;
};

export const Candidates = ({ candidates, gridPosition, showGuide = true }: CandidatesProps) => {
    const base = useRecoilValue(sudokuBaseState);
    const input = useRecoilValue(inputState);
    const hint = useRecoilValue(hintState);

    return (
        <div className="candidates">
            {candidates.map((candidate) => {
                // Candidates are 1 based, grid calculations are 0 based.
                const { column, row } = indexToPosition({ blockIndex: candidate - 1, base });

                const style: CSS.Properties = {
                    "--candidate-column": column,
                    "--candidate-row": row,
                };
                const isGuide = showGuide && input.stickyMode && input.selectedValue === candidate;

                // TODO: optimize
                //  prototype different deductions data structures via selector
                const isDeductionReason = hint?.deductions.some((deduction) =>
                    deduction.reasons.some(
                        (reason) => isEqual(reason.position, gridPosition) && reason.candidates.includes(candidate),
                    ),
                );
                const isDeductionDelete = hint?.deductions.some((deduction) =>
                    deduction.actions.some(
                        (action) =>
                            isEqual(action.position, gridPosition) &&
                            "deleteCandidates" in action &&
                            action.deleteCandidates.includes(candidate),
                    ),
                );

                return (
                    <div
                        className={classNames("candidate", {
                            "candidate--guide": isGuide,
                            "candidate--deduction-reason": isDeductionReason,
                            "candidate--deduction-delete": isDeductionDelete,
                        })}
                        style={style}
                        key={candidate}
                    >
                        {valueToString(candidate)}
                    </div>
                );
            })}
        </div>
    );
};

type CellProps = {
    cell: TransportCell;
    isSelected: boolean;
    isGuide: boolean;
};

export const Cell = (props: CellProps) => {
    const { cell, isSelected, isGuide } = props;

    const { position: gridPosition } = cell;

    const cellClassNames = classNames(
        "cell",
        cellBackgroundClass(isSelected, isGuide),
        cellColorClass(cell.kind === "value" && cell.fixed, cell.incorrectValue),
    );

    const handlePosition = useHandlePosition();

    return (
        <div
            className={cellClassNames}
            onPointerDown={({ buttons, isPrimary, pointerId, pointerType, target }) => {
                if (
                    // Left Mouse, Touch Contact, Pen contact
                    buttons !== 1 ||
                    !isPrimary
                ) {
                    return;
                }
                console.debug("onPointerDown", { isPrimary, buttons, pointerId, pointerType, target });

                // Disable implicit pointer capture, e.g. handle touch events with mouse event semantics
                if ((target as Element).hasPointerCapture(pointerId)) {
                    (target as Element).releasePointerCapture(pointerId);
                }

                if (pointerType !== "touch") {
                    handlePosition(gridPosition).catch(console.error);
                }
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
            {cell.kind === "value" ? (
                <CellValue value={cell.value} />
            ) : (
                <Candidates candidates={cell.candidates} gridPosition={gridPosition} />
            )}
        </div>
    );
};
