import { clamp } from "es-toolkit";
import { useCallback, type KeyboardEvent } from "react";
import { useAtomCallback } from "jotai/utils";
import type { DynamicPosition, TransportSudoku } from "../types";
import { useToggleCandidateMode, useToggleColorMode, useToggleStickyMode } from "./actions/inputActions";
import {
    useDeleteSelectedCell,
    useHandlePosition,
    useHandleValue,
    useSetAllDirectCandidates,
    useUndo,
} from "./actions/sudokuActions";
import { inputState } from "./state/input";
import { sudokuSideLengthState } from "./state/sudoku";

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
    sideLength: TransportSudoku["sideLength"],
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
    const toggleColorMode = useToggleColorMode();

    const onKeyDown: React.KeyboardEventHandler<HTMLDivElement> = useAtomCallback(
        useCallback(
            (get, _set, ev: KeyboardEvent): void => {
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

                    const sideLength = await get(sudokuSideLengthState);
                    const value = keyToValue(key, sideLength);
                    if (value !== undefined) {
                        ev.preventDefault();
                        return await handleValue(value);
                    }

                    const input = get(inputState);
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
                        case "c":
                            ev.preventDefault();
                            toggleColorMode();
                            break;
                        case "Backspace":
                            ev.preventDefault();
                            await undo();
                            break;
                        default:
                            return;
                    }
                };

                asyncEventHandler().catch((err) => {
                    console.error("Error in key down handler", ev, ":", err);
                });
            },
            [
                deleteSelectedCell,
                handlePosition,
                handleValue,
                setAllDirectCandidates,
                toggleCandidateMode,
                toggleColorMode,
                toggleStickyMode,
                undo,
            ],
        ),
    );

    return {
        onKeyDown,
    };
}
