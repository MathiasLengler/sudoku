import { Slider } from "@mui/material";
import classNames from "classnames";
import type * as CSS from "csstype";
import { useAtom, useAtomValue } from "jotai";
import { useAtomCallback } from "jotai/utils";
import * as _ from "lodash-es";
import { memo, useCallback, useDeferredValue, useMemo } from "react";
import { Grid, type CellComponentProps } from "react-window";
import type { Quadrant } from "../../../types";
import { usePlaySelectedGrid } from "../../actions/worldActions";
import { Candidates, CellValue } from "../../grid/cell";
import { sudokuBaseState, sudokuSideLengthState } from "../../state/sudoku";
import {
    cellWorldDimensionsState,
    emptyWasmCellWorldState,
    selectedGridPositionState,
    worldCellSizeState,
    worldCellState,
} from "../../state/world";
import { worldCellBorderClassesState } from "../../state/world/cellBorder";
import { worldCellPositionSchema } from "../../state/world/schema";
import { cellColorClass } from "../../utils/sudoku";

const WorldCellVirtualized = memo(function WorldCellVirtualized({ rowIndex, columnIndex, style }: CellComponentProps) {
    const cellWorldPosition = useMemo(
        () =>
            worldCellPositionSchema.parse({
                row: rowIndex,
                column: columnIndex,
            }),
        [columnIndex, rowIndex],
    );

    const worldCell = useAtomValue(worldCellState(cellWorldPosition));

    const worldCellBorderClasses = useAtomValue(worldCellBorderClassesState(cellWorldPosition));

    const playSelectedGrid = usePlaySelectedGrid();

    const cellOnClick = useAtomCallback(
        useCallback(
            async (get, set, e: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
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

                const emptyCellWorld = await get(emptyWasmCellWorldState);

                const nearestWorldGridCellPosition = emptyCellWorld.worldCellPositionToNearestWorldGridCellPosition(
                    cellWorldPosition,
                    tieBreak,
                );
                const prev = get(selectedGridPositionState);
                const current = nearestWorldGridCellPosition.world_grid_pos;

                if (_.isEqual(prev, current)) {
                    playSelectedGrid().catch(console.error);
                    return;
                }

                set(selectedGridPositionState, current);
            },
            [cellWorldPosition, playSelectedGrid],
        ),
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

function WorldMapVirtualized() {
    const cellWorldDimensions = useAtomValue(cellWorldDimensionsState);
    const worldCellSize = useDeferredValue(useAtomValue(worldCellSizeState));

    return (
        <Grid
            className="world-map-grid"
            columnCount={cellWorldDimensions.cellDim.columnCount}
            columnWidth={worldCellSize}
            rowCount={cellWorldDimensions.cellDim.rowCount}
            rowHeight={worldCellSize}
            cellComponent={WorldCellVirtualized}
            cellProps={{}}
        ></Grid>
    );
}

export function WorldMap() {
    const base = useAtomValue(sudokuBaseState);
    const sideLength = useAtomValue(sudokuSideLengthState);
    const [cellSize, setCellSize] = useAtom(worldCellSizeState);

    const cssVariables: CSS.Properties = {
        "--side-length": sideLength,
        "--base": base,
    };

    return (
        <div className="world-map" style={cssVariables}>
            <Slider min={1} max={200} value={cellSize} onChange={(_e, value) => setCellSize(value)} />
            <WorldMapVirtualized />
        </div>
    );
}
