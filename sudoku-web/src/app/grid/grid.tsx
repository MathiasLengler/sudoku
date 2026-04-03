import type { UseResizeDetectorReturn } from "react-resize-detector";
import { useAtomValue } from "jotai";
import { sudokuBlocksIndexesState, sudokuCellsState } from "../state/sudoku";
import { Block } from "./block";

type GridProps = {
    gridRef: UseResizeDetectorReturn<HTMLDivElement>["ref"];
};

export function Grid({ gridRef }: GridProps) {
    const blocksIndexes = useAtomValue(sudokuBlocksIndexesState);
    const cells = useAtomValue(sudokuCellsState);

    return (
        <div className="grid-container">
            <div className="grid" ref={gridRef}>
                {blocksIndexes.map((cellIndices, blockIndex) => (
                    <Block
                        key={blockIndex}
                        cells={cellIndices.map((cellIndex) => {
                            const cell = cells[cellIndex];
                            if (!cell) {
                                throw new Error(
                                    `index out of bounds: the length is ${cells.length} but the index is ${cellIndex}`,
                                );
                            }
                            return cell;
                        })}
                        blockIndex={blockIndex}
                    />
                ))}
            </div>
        </div>
    );
}
