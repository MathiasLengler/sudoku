import type * as React from "react";
import { useRecoilValue } from "recoil";
import { sudokuBlocksIndexesState, sudokuCellsState } from "../state/sudoku";
import { Block } from "./block";
import type { OnRefChangeType } from "react-resize-detector/build/types/types";

interface GridProps {
    gridRef: OnRefChangeType<HTMLDivElement>;
}

export const Grid = ({ gridRef }: GridProps) => {
    const blocksIndexes = useRecoilValue(sudokuBlocksIndexesState);
    const cells = useRecoilValue(sudokuCellsState);

    return (
        <>
            <div className="grid-container">
                <div className="grid" ref={gridRef}>
                    {blocksIndexes.map((cellIndices, blockIndex) => (
                        <Block
                            key={blockIndex}
                            cells={cellIndices.map((cellIndex) => cells[cellIndex])}
                            blockIndex={blockIndex}
                        />
                    ))}
                </div>
            </div>
        </>
    );
};
