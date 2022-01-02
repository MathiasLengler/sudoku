const KEY_CELLS = "cells_v1";

export function saveCells(cells: Cell[]) {
    localStorage.setItem(KEY_CELLS, JSON.stringify(cells));
}

export function loadCells(): Cell[] | undefined {
    const cellsString = localStorage.getItem(KEY_CELLS);

    if (!cellsString) return undefined;

    let cells;
    try {
        cells = JSON.parse(cellsString);
    } catch (err) {
        console.error("Error while parsing persisted cells:", err);
        return undefined;
    }

    return cells;
}
