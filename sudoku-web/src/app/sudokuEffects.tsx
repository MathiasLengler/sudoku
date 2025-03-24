import { useNotifications } from "@toolpad/core/useNotifications";
import * as _ from "lodash-es";
import { Suspense, useCallback, useEffect } from "react";
import { useRecoilValue, useSetRecoilState } from "recoil";
import type { DynamicCell, DynamicPosition } from "../types";
import { useEndStickyChain } from "./actions/inputActions";
import { saveCells } from "./state/cellsPersistence";
import { inputState } from "./state/input";
import { gameCounterState, sudokuBaseState, sudokuCellsState, sudokuSolutionState } from "./state/sudoku";
import { baseToSideLength } from "./utils/sudoku";

function SaveCellsEffect() {
    const cells = useRecoilValue(sudokuCellsState);

    useEffect(() => {
        saveCells(cells.map(({ position, incorrectValue, ...cell }): DynamicCell => cell));
    }, [cells]);

    return null;
}

function SudokuBaseEffect() {
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
}

function SolutionEffect() {
    const notifications = useNotifications();

    const solution = useRecoilValue(sudokuSolutionState);
    const gameCounter = useRecoilValue(gameCounterState);

    useEffect(() => {
        if (solution === "noSolution") {
            notifications.show("Sudoku has no solutions", {
                key: "no-solution",
                severity: "warning",
            });
        }
        if (solution === "multipleSolutions") {
            notifications.show("Sudoku has multiple solutions", {
                key: "multiple-solutions",
                severity: "warning",
            });
        }
    }, [notifications, solution, gameCounter]);

    return null;
}

function PointerUpHandler() {
    const endStickyChain = useEndStickyChain();

    const onPointerUp = useCallback(
        ({ isPrimary }: PointerEvent): void => {
            if (!isPrimary) {
                return;
            }

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
}

export function SudokuEffects() {
    return (
        <Suspense fallback={null}>
            <SaveCellsEffect />
            <SudokuBaseEffect />
            <SolutionEffect />
            <PointerUpHandler />
        </Suspense>
    );
}
