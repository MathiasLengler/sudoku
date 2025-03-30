import { Slider } from "@mui/material";
import classNames from "classnames";
import type * as CSS from "csstype";
import * as _ from "lodash-es";
import { memo, useDeferredValue, useMemo } from "react";
import AutoSizer from "react-virtualized-auto-sizer";
import { FixedSizeGrid as Grid } from "react-window";
import { useRecoilCallback, useRecoilState, useRecoilValue } from "recoil";
import type { Quadrant } from "../../../types";
import { usePlaySelectedGrid } from "../../actions/worldActions";
import { Candidates, CellValue } from "../../grid/cell";
import { sudokuBaseState, sudokuSideLengthState } from "../../state/sudoku";
import {
    cellWorldDimensionsState,
    emptyWasmCellWorldState,
    selectedGridPositionState,
    worldCellPositionSchema,
    worldCellSizeState,
    worldCellState,
} from "../../state/world";
import { worldCellBorderClassesState } from "../../state/world/cellBorder";
import { cellColorClass } from "../../utils/sudoku";

type WorldCellVirtualizedProps = {
    rowIndex: number;
    columnIndex: number;
    style: React.CSSProperties;
};

const WorldCellVirtualized = memo(function WorldCellVirtualized({
    rowIndex,
    columnIndex,
    style,
}: WorldCellVirtualizedProps) {
    const cellWorldPosition = useMemo(
        () =>
            worldCellPositionSchema.parse({
                row: rowIndex,
                column: columnIndex,
            }),
        [columnIndex, rowIndex],
    );

    const worldCell = useRecoilValue(worldCellState(cellWorldPosition));

    const worldCellBorderClasses = useRecoilValue(worldCellBorderClassesState(cellWorldPosition));

    const playSelectedGrid = usePlaySelectedGrid();

    const cellOnClick = useRecoilCallback(
        ({ snapshot, set }) =>
            async (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
                const { width, height } = e.currentTarget.getBoundingClientRect();
                const centerX = width / 2;
                const centerY = height / 2;
                const clickX = e.nativeEvent.offsetX;
                const clickY = e.nativeEvent.offsetY;

                let tieBreak: Quadrant;
                if (clickX <= centerX && clickY <= centerY) {
                    tieBreak = "topLeft";
                } else if (clickX > centerX && clickY <= centerY) {
                    tieBreak = "topRight";
                } else if (clickX <= centerX && clickY > centerY) {
                    tieBreak = "bottomLeft";
                } else if (clickX > centerX && clickY > centerY) {
                    tieBreak = "bottomRight";
                } else {
                    console.warn("Unexpected click position", { clickX, clickY, width, height });
                    return;
                }

                const emptyCellWorld = await snapshot.getPromise(emptyWasmCellWorldState);

                const nearestWorldGridCellPosition = emptyCellWorld.worldCellPositionToNearestWorldGridCellPosition(
                    cellWorldPosition,
                    tieBreak,
                );

                set(selectedGridPositionState, (prev) => {
                    const current = nearestWorldGridCellPosition.world_grid_pos;

                    if (_.isEqual(prev, current)) {
                        playSelectedGrid().catch(console.error);
                        return prev;
                    }

                    return current;
                });
            },
        [cellWorldPosition, playSelectedGrid],
    );

    const cellClassNames = classNames(
        "cell",
        cellColorClass(
            worldCell.kind === "value" && worldCell.fixed,
            // TODO: incorrectValue for world cell
            //  currently only calculated based on solved grid
            false,
        ),
    );

    return (
        <div className={`world-map-cell ${worldCellBorderClasses}`} style={style} onClick={cellOnClick}>
            <div className={cellClassNames}>
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

export const WorldMap = () => {
    const base = useRecoilValue(sudokuBaseState);
    const sideLength = useRecoilValue(sudokuSideLengthState);
    const [cellSize, setCellSize] = useRecoilState(worldCellSizeState);

    const cssVariables: CSS.Properties = {
        "--side-length": sideLength,
        "--base": base,
    };

    return (
        <div className="world-map" style={cssVariables}>
            <Slider min={1} max={200} value={cellSize} onChange={(_e, value) => setCellSize(value as number)} />
            <WorldMapVirtualized />
        </div>
    );
};
