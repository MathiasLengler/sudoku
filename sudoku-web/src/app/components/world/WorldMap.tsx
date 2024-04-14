import { Button, Slider } from "@mui/material";
import type * as CSS from "csstype";
import React, { useDeferredValue } from "react";
import AutoSizer from "react-virtualized-auto-sizer";
import { FixedSizeGrid as Grid } from "react-window";
import { useRecoilState, useRecoilValue, useResetRecoilState, useSetRecoilState } from "recoil";
import type { DynamicPosition } from "../../../types";
import { Candidates, CellValue } from "../../grid/cell";
import { sudokuBaseState, sudokuSideLengthState } from "../../state/sudoku";
import {
    cellWorldDimensionsState,
    emptyWasmCellWorldState,
    selectedGridIndexState,
    worldCellPositionSchema,
    worldCellSizeState,
    worldCellState,
} from "../../state/world";
import { worldCellBorderClassesState } from "../../state/world/cellBorder";

type WorldCellVirtualizedProps = {
    rowIndex: number;
    columnIndex: number;
    style: React.CSSProperties;
};

const WorldCellVirtualized = React.memo(function WorldCellVirtualized({
    rowIndex,
    columnIndex,
    style,
}: WorldCellVirtualizedProps) {
    const cellWorldPosition: DynamicPosition = {
        row: rowIndex,
        column: columnIndex,
    };

    const worldCell = useRecoilValue(worldCellState(cellWorldPosition));

    const worldCellBorderClasses = useRecoilValue(worldCellBorderClassesState(cellWorldPosition));

    return (
        <div
            className={`world-map-cell ${worldCellBorderClasses}`}
            style={style}
            onClick={(e) => {
                e.currentTarget.scrollIntoView({
                    behavior: "smooth",
                    block: "center",
                    inline: "center",
                });
            }}
        >
            <div className="cell">
                {/* <Code wrap>{debug}</Code> */}
                {worldCell.kind === "value" ? (
                    <CellValue value={worldCell.value} />
                ) : (
                    <Candidates
                        candidates={worldCell.candidates}
                        gridPosition={{ column: 0, row: 0 }}
                        showGuide={false}
                    />
                )}
            </div>
        </div>
    );
});

const WorldMapVirtualized = () => {
    const cellWorldDimensions = useRecoilValue(cellWorldDimensionsState);
    const worldCellSize = useDeferredValue(useRecoilValue(worldCellSizeState));

    return (
        <div className="world-map-grid-auto-sizer-container">
            <AutoSizer className="world-map-grid-auto-sizer">
                {({ height, width }) => (
                    <Grid
                        className="world-map-grid"
                        columnCount={cellWorldDimensions.cellDim.columnCount}
                        columnWidth={worldCellSize}
                        height={height}
                        rowCount={cellWorldDimensions.cellDim.rowCount}
                        rowHeight={worldCellSize}
                        width={width}
                    >
                        {({ columnIndex, rowIndex, style }) => (
                            <WorldCellVirtualized rowIndex={rowIndex} columnIndex={columnIndex} style={style} />
                        )}
                    </Grid>
                )}
            </AutoSizer>
        </div>
    );
};

// TODO: change grid
export const WorldMap = () => {
    // useRecoilValue(isWorkerReadyState);

    const base = useRecoilValue(sudokuBaseState);
    const sideLength = useRecoilValue(sudokuSideLengthState);
    const setSelectedGridIndex = useSetRecoilState(selectedGridIndexState);
    const resetSelectedGridIndex = useResetRecoilState(selectedGridIndexState);
    const [cellSize, setCellSize] = useRecoilState(worldCellSizeState);
    const emptyCellWorld = useRecoilValue(emptyWasmCellWorldState);

    console.log(
        emptyCellWorld.worldCellPositionToNearestWorldGridCellPosition(
            worldCellPositionSchema.parse({ row: 1, column: 1 }),
            "topLeft",
        ),
    );

    const cssVariables: CSS.Properties = {
        "--side-length": sideLength,
        "--base": base,
    };

    return (
        <div className="world-map" style={cssVariables}>
            <Slider min={1} max={200} value={cellSize} onChange={(_e, value) => setCellSize(value as number)} />
            <Button onClick={() => setSelectedGridIndex({ row: 1, column: 2 })}>setSelectedGridIndex</Button>
            <Button onClick={() => resetSelectedGridIndex()}>resetSelectedGridIndex</Button>
            <WorldMapVirtualized />
        </div>
    );
};
