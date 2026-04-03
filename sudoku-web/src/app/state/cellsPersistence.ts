import type { DynamicCells } from "../../types";
import { DynamicCellsSchema } from "./sudoku";

const KEY_CELLS = "cells_v1";

export function saveCells(cells: DynamicCells) {
    console.debug("Saving sudoku cells to localStorage");

    localStorage.setItem(KEY_CELLS, JSON.stringify(cells));
}

export function loadCells(): DynamicCells | undefined {
    const cellsString = localStorage.getItem(KEY_CELLS);
    if (!cellsString) return undefined;

    try {
        return DynamicCellsSchema.parse(JSON.parse(cellsString));
    } catch (err) {
        console.error("Error while parsing persisted cells:", err);
        localStorage.removeItem(KEY_CELLS);
        return undefined;
    }
}
