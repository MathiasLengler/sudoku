import React, { useCallback, useEffect, useMemo } from "react";
import { useEndStickyChain } from "./sudokuActions";
import debounce from "lodash/debounce";
import type { TransportSudoku } from "../../../sudoku-rs/bindings";
import type { CellViews } from "../types";
import { saveCellViews } from "./persistence";
import { useRecoilValue } from "recoil";
import { sudokuCellsState } from "./state/sudoku";

function SaveCellsEffect() {
    const cells = useRecoilValue(sudokuCellsState);

    // TODO: refactor using recoil
    //  use Recoil Sync?
    //  replace with atom effect?
    //  move inside side effect component?
    const debouncedSaveCells = useMemo(
        () =>
            debounce((cells: TransportSudoku["cells"]) => {
                console.debug("Saving sudoku cells to localStorage");
                const cellViews: CellViews = cells.map(({ position, incorrectValue, ...cell }) => cell);
                saveCellViews(cellViews);
            }, 500),
        []
    );

    useEffect(() => {
        debouncedSaveCells(cells);
    }, [debouncedSaveCells, cells]);

    return null;
}

export const PointerUpHandler = () => {
    const endStickyChain = useEndStickyChain();

    const onPointerUp = useCallback(
        ({ isPrimary, buttons, pointerId }: PointerEvent): void => {
            if (!isPrimary) {
                return;
            }
            console.debug("window.onPointerUp", { isPrimary, buttons, pointerId });

            endStickyChain().catch(console.error);
        },
        [endStickyChain]
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

export function SudokuEffects() {
    return (
        <>
            <SaveCellsEffect />
            <PointerUpHandler />
        </>
    );
}
