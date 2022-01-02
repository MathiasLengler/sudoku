import * as React from "react";
import { useEffect, useState } from "react";
import type { TypedWasmSudoku } from "../typedWasmSudoku";
import CircularProgress from "@mui/material/CircularProgress";
import * as Comlink from "comlink";
import { Sudoku } from "./sudoku";
import { WorkerApi } from "../worker";
import { MyTheme } from "./myTheme";
import { Stack, Typography } from "@mui/material";
import { WORKER_BOOT_UP_MESSAGE } from "../constants";
import { loadCells } from "./persistence";

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
                    console.debug("Worker message TX", ev.data);
                });
                worker.addEventListener("error", ev => {
                    console.error("Worker error", ev);
                });
                worker.addEventListener("messageerror", ev => {
                    console.error("Worker messageerror", ev);
                });
            }

            setLoadingStatus("Waiting for worker to load");
            const bootUpMessage = await new Promise<string>((resolve, reject) => {
                const controller = new AbortController();

                worker.addEventListener(
                    "message",
                    (ev: MessageEvent) => {
                        console.debug("workerBootUpListener", ev);

                        // Only capture the first event/message.
                        controller.abort();

                        if (ev.data === WORKER_BOOT_UP_MESSAGE) {
                            resolve(ev.data);
                        } else {
                            reject(new Error(`Unexpected message: ${ev.data}`));
                        }
                    },
                    { signal: controller.signal }
                );
            });

            setLoadingStatus(bootUpMessage);

            const workerApi = Comlink.wrap<WorkerApi>(worker);

            setLoadingStatus("Initializing worker");
            await workerApi.init(loadCells());

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
