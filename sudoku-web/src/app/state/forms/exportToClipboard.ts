import { atom } from "jotai";
import { atomWithStorage } from "jotai/utils";
import * as z from "zod";
import { gridFormatSchema } from "../../constants";
import { getZodLocalStorage } from "../localStorageEffect";
import { wasmSudokuState } from "../mainThread";
import { sudokuState } from "../sudoku";

export type ExportToClipboardFormValues = z.infer<typeof exportToClipboardFormValuesSchema>;
export const exportToClipboardFormValuesSchema = z.object({
    gridFormat: gridFormatSchema,
});

export const EXPORT_TO_CLIPBOARD_FORM_DEFAULT_VALUES = {
    gridFormat: "CandidatesGridPlain",
} satisfies ExportToClipboardFormValues;
export const exportToClipboardFormValuesState = atomWithStorage<ExportToClipboardFormValues>(
    "ExportToClipboardFormValues",
    EXPORT_TO_CLIPBOARD_FORM_DEFAULT_VALUES,
    getZodLocalStorage(exportToClipboardFormValuesSchema),
);

export const exportedGridStringState = atom<Promise<string>>(async (get) => {
    // The exported grid string depends on the sudoku state.
    await get(sudokuState);
    const wasmSudoku = await get(wasmSudokuState);
    const { gridFormat } = get(exportToClipboardFormValuesState);
    return wasmSudoku.export(gridFormat);
});
