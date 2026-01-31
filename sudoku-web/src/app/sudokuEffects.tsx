import { notifications } from "@mantine/notifications";
import * as _ from "es-toolkit";
import { Suspense, useCallback, useEffect } from "react";
import { useAtomValue, useSetAtom } from "jotai";
import type { DynamicCell, DynamicPosition } from "../types";
import { useEndStickyChain } from "./actions/inputActions";
import { saveCells } from "./state/cellsPersistence";
import { inputState } from "./state/input";
import { gameCounterState, sudokuBaseState, sudokuCellsState, sudokuSolutionState } from "./state/sudoku";
import { baseToSideLength } from "./utils/sudoku";

function SaveCellsEffect() {
    const cells = useAtomValue(sudokuCellsState);

    useEffect(() => {
        saveCells(cells.map(({ position, incorrectValue, ...cell }): DynamicCell => cell));
    }, [cells]);

    return null;
}

function SudokuBaseEffect() {
    const base = useAtomValue(sudokuBaseState);
    const setInput = useSetAtom(inputState);

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
    const solution = useAtomValue(sudokuSolutionState);
    const gameCounter = useAtomValue(gameCounterState);

    useEffect(() => {
        if (solution === "noSolution") {
            notifications.show({
                id: "no-solution",
                title: "Sudoku has no solutions",
                message: "",
                color: "yellow",
            });
        }
        if (solution === "multipleSolutions") {
            notifications.show({
                id: "multiple-solutions",
                title: "Sudoku has multiple solutions",
                message: "",
                color: "yellow",
            });
        }
    }, [solution, gameCounter]);

    return null;
}

function PointerUpHandler() {
    const endStickyChain = useEndStickyChain();

    const onPointerUp = useCallback(
        ({ isPrimary }: PointerEvent): void => {
            if (!isPrimary) {
                return;
            }

            endStickyChain();
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
