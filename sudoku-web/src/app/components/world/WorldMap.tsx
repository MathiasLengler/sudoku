import { Button, Slider } from "@mui/material";
import Box from "@mui/material/Box";
import classNames from "classnames";
import type * as CSS from "csstype";
import _ from "lodash";
import React, { useDeferredValue } from "react";
import { useRecoilState, useRecoilValue, useResetRecoilState, useSetRecoilState } from "recoil";
import type { DynamicCell, DynamicPosition } from "../../../types";
import { Candidates, CellValue } from "../../grid/cell";
import { sudokuBaseState, sudokuSideLengthState } from "../../state/sudoku";
import {
    allWorldCellsState,
    cellWorldDimensionsState,
    selectedGridIndexState,
    worldCellSizeState,
} from "../../state/world";
import { worldCellBorderClassesState } from "../../state/world/cellBorder";

type WorldCellProps = {
    rowIndex: number;
    columnIndex: number;
    worldCell: DynamicCell;
};

// TODO: naming: grid vs tile

export const WorldCellMemo = React.memo(function WorldCell({ rowIndex, columnIndex, worldCell }: WorldCellProps) {
    console.log("WorldCell", rowIndex, columnIndex);

    const cellWorldPosition: DynamicPosition = {
        row: rowIndex,
        column: columnIndex,
    };

    const worldCellBorderClasses = useRecoilValue(worldCellBorderClassesState(cellWorldPosition));

    return (
        <td className={worldCellBorderClasses}>
            <Box
                className="cell"
                sx={{
                    width: "var(--cell-size)",
                    height: "var(--cell-size)",
                }}
                onClick={(e) => {
                    e.currentTarget.scrollIntoView({
                        behavior: "smooth",
                        block: "center",
                        inline: "center",
                    });
                }}
            >
                {/* <Code wrap>{debug}</Code> */}
                {worldCell.kind === "value" ? (
                    <CellValue value={worldCell.value} />
                ) : (
                    <Candidates candidates={worldCell.candidates} gridPosition={{ column: 0, row: 0 }} />
                )}
            </Box>
        </td>
    );
});

export const WorldMapTableMemo = React.memo(function WorldMapTable() {
    const allWorldCells = useRecoilValue(allWorldCellsState);
    const cellWorldDimensions = useRecoilValue(cellWorldDimensionsState);

    return (
        <table className="world-map-table">
            <tbody>
                {_.chunk(allWorldCells, cellWorldDimensions.cellDim.columnCount).map((row, rowIndex) => (
                    <tr key={rowIndex}>
                        {row.map((cell, columnIndex) => (
                            <WorldCellMemo
                                key={columnIndex}
                                worldCell={cell}
                                columnIndex={columnIndex}
                                rowIndex={rowIndex}
                            />
                        ))}
                    </tr>
                ))}
            </tbody>
        </table>
    );
});

// TODO: change grid
export const WorldMap = () => {
    const base = useRecoilValue(sudokuBaseState);
    const sideLength = useRecoilValue(sudokuSideLengthState);
    const setSelectedGridIndex = useSetRecoilState(selectedGridIndexState);
    const resetSelectedGridIndex = useResetRecoilState(selectedGridIndexState);

    const [cellSize, setCellSize] = useRecoilState(worldCellSizeState);

    const cellSizeCss = `${useDeferredValue(cellSize)}px`;

    const cssVariables: CSS.Properties = {
        "--side-length": sideLength,
        "--base": base,
        "--cell-size": cellSizeCss,
    };

    return (
        <div className="world-map" style={cssVariables}>
            <Slider min={1} max={200} value={cellSize} onChange={(_e, value) => setCellSize(value as number)} />
            <Button onClick={() => setSelectedGridIndex({ row: 1, column: 2 })}>setSelectedGridIndex</Button>
            <Button onClick={() => resetSelectedGridIndex()}>resetSelectedGridIndex</Button>
            <WorldMapTableMemo />
        </div>
    );
};
