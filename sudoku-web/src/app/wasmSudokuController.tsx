import { CellPosition, GeneratorSettings, GridFormat, TransportCell, TransportSudoku, WasmSudoku } from "../types";
import * as React from "react";
import { blocksToCell } from "./utils";
import isEqual from "lodash/isEqual";
import * as Comlink from "comlink";
import type { StrategyName } from "../../../sudoku-wasm/pkg";

export type onSudokuUpdate = (this: void, sudoku: TransportSudoku) => void;

export interface Input {
    stickyMode: boolean;
    candidateMode: boolean;
    selectedPos: CellPosition;
    selectedCell: TransportCell;
    // FIXME: only useful if stickyMode is true
    selectedValue: number;
}

export class WasmSudokuController {
    public constructor(
        private readonly wasmSudokuProxy: Comlink.Remote<WasmSudoku>,
        private readonly sudoku: TransportSudoku,
        private readonly onSudokuUpdate: onSudokuUpdate,
        private readonly input: Input,
        private readonly setInput: React.Dispatch<React.SetStateAction<Omit<Input, "selectedCell">>>,
        private readonly sideLength: TransportSudoku["sideLength"]
    ) {}

    private async updateSudoku() {
        const transportSudoku = await this.wasmSudokuProxy.getSudoku();
        this.onSudokuUpdate(transportSudoku);
    }

    private async withSudokuUpdate<T>(f: () => Promise<T>): Promise<T> {
        const ret = await f();

        await this.updateSudoku();

        return ret;
    }

    private checkFixed(): boolean {
        const selectedCell = this.input.selectedCell;
        if (selectedCell.kind === "value" && selectedCell.fixed) {
            console.warn("WasmSudokuController", "cannot modify a fixed cell", selectedCell);

            return true;
        } else {
            return false;
        }
    }

    public async handlePosition(newSelectedPosition: CellPosition, move = false): Promise<void> {
        const { stickyMode, selectedPos, selectedValue } = this.input;

        if (move && isEqual(selectedPos, newSelectedPosition)) {
            return;
        }

        this.setSelectedPosition(newSelectedPosition);

        if (stickyMode) {
            await this.setSelectedCell(selectedValue);
        }
    }

    private setSelectedPosition(selectedPos: CellPosition) {
        const {
            sudoku: { base, blocks },
        } = this;
        const selectedCell = blocksToCell(blocks, selectedPos, base);

        this.setInput(prevInput => ({ ...prevInput, selectedPos }));
        this.input.selectedPos = selectedPos;
        this.input.selectedCell = selectedCell;
    }

    public async handleValue(value: number): Promise<void> {
        const { stickyMode } = this.input;

        if (value > this.sideLength) {
            console.warn("WasmSudokuController", "tried to handle value greater than current sudoku allows:", value);

            return;
        }

        if (stickyMode) {
            this.setInput(prevInput => ({ ...prevInput, selectedValue: value }));
        } else {
            await this.setSelectedCell(value);
        }
    }

    private async setSelectedCell(value: number): Promise<void> {
        const { candidateMode, selectedPos } = this.input;

        if (this.checkFixed()) {
            return;
        }

        await this.withSudokuUpdate(async () => {
            if (value === 0) {
                await this.wasmSudokuProxy.delete(selectedPos);
            } else {
                if (candidateMode) {
                    await this.wasmSudokuProxy.toggleCandidate(selectedPos, value);
                } else {
                    await this.wasmSudokuProxy.setOrToggleValue(selectedPos, value);
                }
            }
        });
    }

    public async delete(): Promise<void> {
        if (this.checkFixed()) {
            return;
        }

        await this.withSudokuUpdate(async () => {
            await this.wasmSudokuProxy.delete(this.input.selectedPos);
        });
    }

    public async setAllDirectCandidates(): Promise<void> {
        await this.withSudokuUpdate(async () => {
            await this.wasmSudokuProxy.setAllDirectCandidates();
        });
    }

    public async undo(): Promise<void> {
        await this.withSudokuUpdate(async () => {
            await this.wasmSudokuProxy.undo();
        });
    }

    public async generate(settings: GeneratorSettings): Promise<void> {
        await this.withSudokuUpdate(async () => {
            await this.wasmSudokuProxy.generate(settings);
        });
    }

    public async import(input: string): Promise<void> {
        await this.withSudokuUpdate(async () => {
            await this.wasmSudokuProxy.import(input);
        });
    }

    public async export(format: GridFormat): Promise<string> {
        return await this.wasmSudokuProxy.export(format);
    }

    public async tryStrategy(strategyName: StrategyName): Promise<boolean> {
        return await this.withSudokuUpdate(async () => {
            return await this.wasmSudokuProxy.tryStrategy(strategyName);
        });
    }

    public async solveSingleCandidates(): Promise<boolean> {
        return await this.withSudokuUpdate(async () => {
            return await this.wasmSudokuProxy.solveSingleCandidates();
        });
    }

    public async groupReduction(): Promise<void> {
        await this.withSudokuUpdate(async () => {
            await this.wasmSudokuProxy.groupReduction();
        });
    }

    public toggleCandidateMode(): void {
        this.setInput(prevInput => ({
            ...prevInput,
            candidateMode: !prevInput.candidateMode,
        }));
    }

    public toggleStickyMode(): void {
        this.setInput(prevInput => ({
            ...prevInput,
            stickyMode: !prevInput.stickyMode,
        }));
    }
}
