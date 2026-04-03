import * as z from "zod";

export type SerializedDynamicCellWorld = z.infer<typeof serializedDynamicCellWorldSchema>;
export const serializedDynamicCellWorldSchema = z.instanceof(Uint8Array).brand<"SerializedDynamicCellWorld">();

export type SerializedDynamicSudoku = z.infer<typeof serializedDynamicSudokuSchema>;
export const serializedDynamicSudokuSchema = z.instanceof(Uint8Array).brand<"SerializedDynamicSudoku">();
