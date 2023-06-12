import type * as React from "react";
import type { KeyboardEvent } from "react";
import clamp from "lodash/clamp";
import type { DynamicPosition, TransportSudoku } from "../types";
import { inputState } from "./state/input";
import { useRecoilCallback } from "recoil";
import {
    useDeleteSelectedCell,
    useHandlePosition,
    useHandleValue,
    useSetAllDirectCandidates,
    useUndo,
} from "./actions/sudokuActions";
import { sudokuSideLengthState } from "./state/sudoku";
import { useToggleCandidateMode, useToggleStickyMode } from "./actions/inputActions";

function keyToValue(key: string, sideLength: number): number | undefined {
    if (key.length === 1) {
        const value = parseInt(key, 36);

        if (Number.isInteger(value) && value <= sideLength) {
            return value;
        }
    }
}

function keyToNewPos(
    key: string,
    selectedPos: DynamicPosition,
    sideLength: TransportSudoku["sideLength"]
): DynamicPosition | undefined {
    let { column, row } = selectedPos;
    switch (key) {
        case "ArrowUp":
            row -= 1;
            break;
        case "ArrowDown":
            row += 1;
            break;
        case "ArrowLeft":
            column -= 1;
            break;
        case "ArrowRight":
            column += 1;
            break;
        default:
            return;
    }

    column = clamp(column, 0, sideLength - 1);
    row = clamp(row, 0, sideLength - 1);

    return { row: row, column: column };
}

export function useKeyboardInput() {
    const handlePosition = useHandlePosition();
    const handleValue = useHandleValue();
    const deleteSelectedCell = useDeleteSelectedCell();
    const setAllDirectCandidates = useSetAllDirectCandidates();
    const undo = useUndo();
    const toggleCandidateMode = useToggleCandidateMode();
    const toggleStickyMode = useToggleStickyMode();

    const onKeyDown: React.KeyboardEventHandler<HTMLDivElement> = useRecoilCallback(
        ({ snapshot }) =>
            (ev: KeyboardEvent): void => {
                const asyncEventHandler = async () => {
                    // TODO: process modifier keys
                    //  shift+backspace => redo
                    //  ctrl+z => undo
                    //  ctrl+shift+z => redo
                    //  ctrl+y => redo
                    const { key, altKey, ctrlKey, metaKey, shiftKey } = ev;
                    if (altKey || ctrlKey || metaKey || shiftKey) {
                        return;
                    }

                    const sideLength = await snapshot.getPromise(sudokuSideLengthState);
                    const value = keyToValue(key, sideLength);
                    if (value !== undefined) {
                        ev.preventDefault();
                        return await handleValue(value);
                    }

                    const input = await snapshot.getPromise(inputState);
                    if (!input.stickyMode) {
                        const newPos = keyToNewPos(key, input.selectedPos, sideLength);
                        if (newPos !== undefined) {
                            ev.preventDefault();
                            return await handlePosition(newPos);
                        }
                    }

                    switch (key) {
                        case " ":
                            ev.preventDefault();
                            toggleCandidateMode();
                            break;
                        case "Delete":
                            ev.preventDefault();
                            await deleteSelectedCell();
                            break;
                        case "Insert":
                            ev.preventDefault();
                            await setAllDirectCandidates();
                            break;
                        case "+":
                            ev.preventDefault();
                            toggleStickyMode();
                            break;
                        case "Backspace":
                            ev.preventDefault();
                            await undo();
                            break;
                        default:
                            return;
                    }
                };

                asyncEventHandler().catch(err => {
                    console.error("Error in key down handler", ev, ":", err);
                });
            },
        [
            deleteSelectedCell,
            handlePosition,
            handleValue,
            setAllDirectCandidates,
            toggleCandidateMode,
            toggleStickyMode,
            undo,
        ]
    );

    return {
        onKeyDown,
    };
}
