import { describe } from "vitest";

describe("worker", () => {
    // TODO: test and bench worker communication
    // Content: Grid, TransportSudoku, CellWorld
    //  each serialized with the most efficient method
    // Channel:
    //  baseline no worker
    //  comlink proxied class return (return structured cloned)
    //  comlink proxied class return (return transferred Uint8Array)
    // Goal: find new architecture: which parts are executed into worker, which parts stay on main thread?
    //  Probably: only heavy computations in worker, light computations on main thread
    //  We will need to manage multiple class instances, transferring the state between main thread and worker depending on the operation.
});
