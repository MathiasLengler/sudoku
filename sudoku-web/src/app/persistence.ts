import type { CellViews } from "../types";

const KEY_CELLS = "cells_v1";

export function saveCellViews(cells: CellViews) {
    localStorage.setItem(KEY_CELLS, JSON.stringify(cells));
}

export function loadCellViews(): CellViews | undefined {
    const cellsString = localStorage.getItem(KEY_CELLS);

    if (!cellsString) return undefined;

    let cellViews;
    try {
        cellViews = JSON.parse(cellsString);
    } catch (err) {
        console.error("Error while parsing persisted cellViews:", err);
        return undefined;
    }

    return cellViews;
}
