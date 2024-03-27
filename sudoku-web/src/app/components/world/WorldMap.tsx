import Box from "@mui/material/Box";
import classnames from "classnames";
import type * as CSS from "csstype";
import { useRecoilValue } from "recoil";
import { Candidates, CellValue, cellColorClass } from "../../grid/cell";
import { sudokuBaseState, sudokuSideLengthState } from "../../state/sudoku";
import { allWorldCellsState, cellWorldDimensionsState } from "../../state/world";
import { Code } from "../Code";

export const WorldMap = () => {
    const allWorldCells = useRecoilValue(allWorldCellsState);
    const cellWorldDimensions = useRecoilValue(cellWorldDimensionsState);
    const base = useRecoilValue(sudokuBaseState);
    const sideLength = useRecoilValue(sudokuSideLengthState);

    // TODO: render tile boundaries
    // TODO: change tile

    const cellSize = "4rem";

    const cssVariables: CSS.Properties = {
        "--side-length": sideLength,
        "--base": base,
    };

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
            <Box
                sx={{
                    display: "grid",
                    gridTemplateRows: `repeat(${cellWorldDimensions.cell_dim.row_count}, ${cellSize})`,
                    gridTemplateColumns: `repeat(${cellWorldDimensions.cell_dim.column_count}, ${cellSize})`,
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
                                "--cell-size": `${cellSize}`,
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
