import Box from "@mui/material/Box";
import classnames from "classnames";
import type * as CSS from "csstype";
import { useRecoilValue } from "recoil";
import { Candidates, CellValue, cellColorClass } from "../../grid/cell";
import { sudokuBaseState, sudokuSideLengthState } from "../../state/sudoku";
import { allWorldCellsState, cellWorldDimensionsState } from "../../state/world";
import { Code } from "../Code";
import { Slider } from "@mui/material";
import { useState, useDeferredValue } from "react";

export const WorldMap = () => {
    const allWorldCells = useRecoilValue(allWorldCellsState);
    const cellWorldDimensions = useRecoilValue(cellWorldDimensionsState);
    const base = useRecoilValue(sudokuBaseState);
    const sideLength = useRecoilValue(sudokuSideLengthState);

    // TODO: render tile boundaries
    // TODO: change tile

    const [cellSize, setCellSize] = useState(100);

    const cssVariables: CSS.Properties = {
        "--side-length": sideLength,
        "--base": base,
    };

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
                sx={{
                    display: "grid",
                    gridTemplateRows: `repeat(${cellWorldDimensions.cell_dim.row_count}, ${cellSizeCss})`,
                    gridTemplateColumns: `repeat(${cellWorldDimensions.cell_dim.column_count}, ${cellSizeCss})`,
                    "--cell-size": `${cellSizeCss}`,
                    overflow: "auto",
                    gap: "1px",
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
            </Box>
        </Box>
    );
};
