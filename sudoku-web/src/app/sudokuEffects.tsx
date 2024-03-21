import React, { useCallback, useEffect } from "react";
import type { DynamicCell, DynamicPosition } from "../types";
import { saveCells } from "./cellsPersistence";
import { useRecoilValue, useSetRecoilState } from "recoil";
import { sudokuBaseState, sudokuCellsState } from "./state/sudoku";
import { inputState } from "./state/input";
import { baseToSideLength } from "./utils";
import _ from "lodash";
import { useEndStickyChain } from "./actions/inputActions";

function SaveCellsEffect() {
    const cells = useRecoilValue(sudokuCellsState);

    useEffect(() => {
        saveCells(cells.map(({ position, incorrectValue, ...cell }): DynamicCell => cell));
    }, [cells]);

    return null;
}

const PointerUpHandler = () => {
    const endStickyChain = useEndStickyChain();

    const onPointerUp = useCallback(
        ({ isPrimary, buttons, pointerId }: PointerEvent): void => {
            if (!isPrimary) {
                return;
            }
            // console.debug("window.onPointerUp", { isPrimary, buttons, pointerId });

            endStickyChain().catch(console.error);
        },
        [endStickyChain],
    );

    useEffect(() => {
        const controller = new AbortController();
        // Listen on window to catch primary pointer transition to inactive outside the cell/grid/window.
        window.addEventListener("pointerup", onPointerUp, { signal: controller.signal });

        return () => {
            controller.abort();
        };
    }, [onPointerUp]);

    return null;
};

const SudokuBaseEffect = () => {
    const base = useRecoilValue(sudokuBaseState);
    const setInput = useSetRecoilState(inputState);

    useEffect(() => {
        setInput((input) => {
            const sideLength = baseToSideLength(base);
            const clampValue = (value: number) => _.clamp(value, sideLength);
            const clampCoordinate = (coordinate: number) => _.clamp(coordinate, sideLength - 1);
            const clampPosition = (pos: DynamicPosition): DynamicPosition => ({
                row: clampCoordinate(pos.row),
                column: clampCoordinate(pos.column),
            });

            if (input.stickyMode) {
                return {
                    ...input,
                    selectedValue: clampValue(input.selectedValue),
                    previouslySelectedPos: clampPosition(input.previouslySelectedPos),
                };
            } else {
                return {
                    ...input,
                    selectedPos: clampPosition(input.selectedPos),
                    previouslySelectedValue: clampValue(input.previouslySelectedValue),
                };
            }
        });
    }, [base, setInput]);

    return null;
};

export function SudokuEffects() {
    return (
        <>
            <SaveCellsEffect />
            <SudokuBaseEffect />
            <PointerUpHandler />
        </>
    );
}
