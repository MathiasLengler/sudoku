import { z } from "zod";
import { gridFormatSchema } from "../../../constants";
import { atom, selector } from "recoil";
import { localStorageEffect } from "../localStorageEffect";
import { sudokuState } from "../sudoku";
import { remoteWorkerApiState } from "../worker";

export type ExportToClipboardFormValues = z.infer<typeof exportToClipboardFormValuesSchema>;
export const exportToClipboardFormValuesSchema = z.object({
    gridFormat: gridFormatSchema,
});

export const EXPORT_TO_CLIPBOARD_FORM_DEFAULT_VALUES = {
    gridFormat: "CandidatesGridPlain",
} satisfies ExportToClipboardFormValues;
export const exportToClipboardFormValuesState = atom<ExportToClipboardFormValues>({
    key: "ExportToClipboardFormValues",
    default: EXPORT_TO_CLIPBOARD_FORM_DEFAULT_VALUES,
    effects: [localStorageEffect(exportToClipboardFormValuesSchema)],
});

export const exportedGridStringState = selector<string>({
    key: "ExportedGridString",
    get: async ({ get }) => {
        // The exported grid string depends on the sudoku state.
        get(sudokuState);
        const wasmSudokuProxy = get(remoteWorkerApiState).wasmSudokuProxy;
        const { gridFormat } = get(exportToClipboardFormValuesState);
        return await wasmSudokuProxy.export(gridFormat);
    },
});
