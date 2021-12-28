import * as React from "react";
import { useEffect, useState } from "react";
import { TypedWasmSudoku } from "../typedWasmSudoku";
import CircularProgress from "@mui/material/CircularProgress";
import * as Comlink from "comlink";
import { Sudoku } from "./sudoku";
import { WorkerApi } from "../worker";
import { MyTheme } from "./myTheme";
import { Stack, Typography } from "@mui/material";

export const App: React.FunctionComponent = () => {
    const [loadingStatus, setLoadingStatus] = useState<string>("Startup");
    const [sudoku, setSudoku] = useState<TransportSudoku | undefined>(undefined);
    const [wasmSudokuProxy, setWasmSudokuProxy] = useState<Comlink.Remote<TypedWasmSudoku> | undefined>(undefined);

    useEffect(() => {
        async function loadSudoku() {
            setLoadingStatus("Creating worker");
            const worker = new Worker(new URL("../worker.tsx", import.meta.url));
            if (process.env.NODE_ENV !== "production") {
                worker.addEventListener("message", ev => {
                    console.debug("Worker TX", ev.data);
                });
            }
            const workerApi = Comlink.wrap<WorkerApi>(worker);

            setLoadingStatus("Initializing worker");
            await workerApi.init();

            setLoadingStatus("Connecting to worker");
            const wasmSudokuProxy = workerApi.typedWasmSudoku as unknown as Comlink.Remote<TypedWasmSudoku>;
            // Important: setState using setter function.
            //  This ensures that React does not misinterpret the comlink proxy instance as a setter function itself.
            //  Otherwise, a rejected promise of an attempt to call the proxy as a function would get set as the state.
            setWasmSudokuProxy(() => wasmSudokuProxy);

            setLoadingStatus("Fetching initial sudoku");
            const transportSudoku = await wasmSudokuProxy.getSudoku();
            setSudoku(transportSudoku);

            setLoadingStatus("Done");
        }

        (async () => {
            try {
                await loadSudoku();
            } catch (err) {
                console.error(err);
                setLoadingStatus(`Unexpected error: ${err}`);
            }
        })();
    }, []);

    let content;

    if (!sudoku || !wasmSudokuProxy) {
        content = (
            <div className="sudoku">
                <Stack direction="column" justifyContent="center" alignItems="center" spacing={2}>
                    <CircularProgress />
                    <Typography>{loadingStatus}</Typography>
                </Stack>
            </div>
        );
    } else {
        content = <Sudoku sudoku={sudoku} setSudoku={setSudoku} wasmSudokuProxy={wasmSudokuProxy} />;
    }

    return <MyTheme>{content}</MyTheme>;
};
