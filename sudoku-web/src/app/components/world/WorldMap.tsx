import { Slider } from "@mui/material";
import Box from "@mui/material/Box";
import type * as CSS from "csstype";
import _ from "lodash";
import { useDeferredValue, useState } from "react";
import { useRecoilValue } from "recoil";
import type { CellWorldDimensions, DynamicPosition } from "../../../types";
import { sudokuBaseState, sudokuSideLengthState } from "../../state/sudoku";
import { allWorldCellsState, cellWorldDimensionsState } from "../../state/world";
import { Code } from "../Code";
import { Candidates, CellValue } from "../../grid/cell";

type GridBorderMarker = "grid";
type BlockBorderMarker = "block";
type BorderMarker = GridBorderMarker | BlockBorderMarker;

type CellBorders<T> = {
    top?: T;
    right?: T;
    bottom?: T;
    left?: T;
};

type AxisBorders<T> = {
    start?: T;
    end?: T;
};

// TODO: test
function getAxisBorders(
    // current cell
    worldAxisIndex: number,
    // world
    cellAxisCount: number,
    overlap: number,
    // grid
    base: number,
    gridSideLength: number,
): { axisBorders: AxisBorders<BorderMarker>; debug: unknown } {
    const gridSideLengthEndIndex = gridSideLength - 1;
    const gridStride = gridSideLength - overlap;
    const gridStrideEndIndex = gridStride - 1;
    const blockStride = base;
    const blockStrideEndIndex = blockStride - 1;

    const hasTileBefore = worldAxisIndex >= gridStride;
    const hasTileAfter = worldAxisIndex < cellAxisCount - gridStride;

    const axisBorders: AxisBorders<BorderMarker> = {};

    const gridAxis = worldAxisIndex % gridStride;
    const blockIndex = Math.floor(gridAxis / blockStride);
    const blockAxis = gridAxis % blockStride;

    if ((gridAxis === 0 && hasTileAfter) || (gridAxis === overlap && hasTileBefore)) {
        axisBorders.start = "grid";
    }
    if (
        gridAxis === gridSideLengthEndIndex ||
        (gridAxis === gridStrideEndIndex && hasTileAfter) ||
        (gridAxis === overlap - 1 && hasTileBefore)
    ) {
        axisBorders.end = "grid";
    }
    if (blockAxis === 0 && blockIndex > 0) {
        axisBorders.start ??= "block";
    } else if (blockAxis === blockStrideEndIndex) {
        axisBorders.end ??= "block";
    }

    return { axisBorders, debug: "" };
}

function getCellBorders(
    {
        overlap,
        tileDim: { rowCount: tileRowCount, columnCount: tileColumnCount },
        cellDim: { rowCount: cellRowCount, columnCount: cellColumnCount },
        tileDim,
        cellDim,
    }: CellWorldDimensions,
    cellIndex: DynamicPosition,
    base: number,
    gridSideLength: number,
): {
    cellBorders: CellBorders<BorderMarker>;
    debug: string;
} {
    const { row: rowIndex, column: columnIndex } = cellIndex;

    if (!(_.inRange(rowIndex, 0, cellRowCount) && _.inRange(columnIndex, 0, cellColumnCount))) {
        throw new Error(
            `cellIndex out of bounds: ${JSON.stringify(cellIndex)} for cellDim: ${JSON.stringify(cellDim)}`,
        );
    }

    const { axisBorders: rowBorders } = getAxisBorders(rowIndex, cellRowCount, overlap, base, gridSideLength);
    const { axisBorders: columnBorders, debug: axisDebug } = getAxisBorders(
        columnIndex,
        cellColumnCount,
        overlap,
        base,
        gridSideLength,
    );

    return {
        cellBorders: _.omitBy(
            {
                top: rowBorders.start,
                right: columnBorders.end,
                bottom: rowBorders.end,
                left: columnBorders.start,
            },
            // eslint-disable-next-line @typescript-eslint/unbound-method
            _.isUndefined,
        ),
        debug: _.chain({ axisDebug })
            .entries()
            .map(([key, value]) => `${key}:${JSON.stringify(value)}`)
            .join("\n")
            .value(),
    };
}

export const WorldMap = () => {
    const allWorldCells = useRecoilValue(allWorldCellsState);
    const cellWorldDimensions = useRecoilValue(cellWorldDimensionsState);
    const base = useRecoilValue(sudokuBaseState);
    const sideLength = useRecoilValue(sudokuSideLengthState);

    // TODO: change tile

    const [cellSize, setCellSize] = useState(100);

    const cssVariables: CSS.Properties = {
        "--side-length": sideLength,
        "--base": base,
    };

    const gridBorder = "11px solid red";
    const blockBorder = "10px solid blue";
    const cellBorder = "5px solid black";

    const cellSizeCss = `${useDeferredValue(cellSize)}px`;

    return (
        <Box
            className="world-map"
            sx={{
                display: "flex",
                flexDirection: "column",
                height: 1,
                ...cssVariables,
            }}
        >
            <Code wrap>{JSON.stringify(cellWorldDimensions)}</Code>
            <Slider min={1} max={200} value={cellSize} onChange={(_e, value) => setCellSize(value as number)} />
            <Box
                component="table"
                sx={{
                    overflow: "auto",
                    display: "block",
                    borderCollapse: "collapse",
                    borderStyle: "groove",
                    "--cell-size": cellSizeCss,
                }}
            >
                <tbody>
                    {_.chunk(allWorldCells, cellWorldDimensions.cellDim.columnCount).map((row, rowIndex) => (
                        <tr key={rowIndex}>
                            {row.map((cell, columnIndex) => {
                                const cellIndex: DynamicPosition = {
                                    column: columnIndex,
                                    row: rowIndex,
                                };

                                const { cellBorders, debug } = getCellBorders(
                                    cellWorldDimensions,
                                    cellIndex,
                                    base,
                                    sideLength,
                                );

                                const cellCssBorders = _.chain(cellBorders)
                                    .mapValues((value) => {
                                        if (value === "grid") {
                                            return gridBorder;
                                        } else if (value === "block") {
                                            return blockBorder;
                                        }
                                    })
                                    .mapKeys((_value, key) => {
                                        return `border${_.capitalize(key)}`;
                                    })
                                    .value();

                                const style = {
                                    border: cellBorder,
                                    padding: 0,
                                    ...cellCssBorders,
                                };

                                return (
                                    <Box component="td" key={columnIndex} sx={style}>
                                        <Box
                                            className="cell"
                                            sx={{
                                                width: cellSizeCss,
                                                height: cellSizeCss,
                                            }}
                                        >
                                            {/* <Code wrap>{debug}</Code> */}
                                            {cell.kind === "value" ? (
                                                <CellValue value={cell.value} />
                                            ) : (
                                                <Candidates
                                                    candidates={cell.candidates}
                                                    gridPosition={{ column: 0, row: 0 }}
                                                />
                                            )}
                                        </Box>
                                    </Box>
                                );
                            })}
                        </tr>
                    ))}
                </tbody>
            </Box>
            {/* <Box
                sx={{
                    display: "grid",
                    gridTemplateRows: `repeat(${cellWorldDimensions.cellDim.rowCount}, ${cellSizeCss})`,
                    gridTemplateColumns: `repeat(${cellWorldDimensions.cellDim.columnCount}, ${cellSizeCss})`,
                    "--cell-size": cellSizeCss,
                    overflow: "auto",
                    gap: "10px",
                    background: "var(--cell-border-color)",
                }}
            >
                {allWorldCells.map((cell, index) => {
                    const cellClassNames = classnames(
                        "cell",
                        cellColorClass(cell.kind === "value" && cell.fixed, false),
                    );
                    return (
                        <Box
                            onClick={(e) => {
                                e.currentTarget.scrollIntoView({
                                    behavior: "smooth",
                                    block: "center",
                                    inline: "center",
                                });
                            }}
                            key={index}
                            className={cellClassNames}
                            sx={{
                                aspectRatio: 1,
                            }}
                        >
                            {cell.kind === "value" ? (
                                <CellValue value={cell.value} />
                            ) : (
                                <Candidates candidates={cell.candidates} gridPosition={{ column: 0, row: 0 }} />
                            )}
                        </Box>
                    );
                })}
            </Box> */}
        </Box>
    );
};
