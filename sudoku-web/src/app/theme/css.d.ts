// TS declaration merging
import "csstype";

declare module "csstype" {
    // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
    interface Properties {
        "--base"?: number;
        "--side-length"?: number;
        "--candidate-column"?: number;
        "--candidate-row"?: number;
        "--block-column"?: number;
        "--block-row"?: number;
        "--grid-size"?: string;
        "--cell-size"?: string;
    }
}
