import * as React from "react";
import {useEffect, useState} from "react";
import {TypedWasmSudoku} from "../typedWasmSudoku";
import CircularProgress from "@mui/material/CircularProgress";
import * as Comlink from "comlink";
import {Theme} from "./theme";
import {Sudoku} from "./sudoku";
import {WorkerApi} from "../worker";
import isEqual from "lodash/isEqual";
import {MyTheme} from "./myTheme";

const defaultTransportSudoku: TransportSudoku = {
  blocks: [[]], base: 1, sideLength: 1, cellCount: 1
};


export const App: React.FunctionComponent = () => {
  // State
  const [loadingStatus, setLoadingStatus] = useState<string>("Startup");
  const [sudoku, setSudoku] = useState<TransportSudoku | undefined>(undefined);
  const [wasmSudokuProxy, setWasmSudokuProxy] = useState<Comlink.Remote<TypedWasmSudoku> | undefined>(undefined);

  useEffect(() => {
    (async () => {
      setLoadingStatus("Creating worker");

      const worker = new Worker(new URL("../worker.tsx", import.meta.url));
      const workerApi = Comlink.wrap<WorkerApi>(worker);

      setLoadingStatus("Initializing worker");
      console.debug(await workerApi.init());
      setLoadingStatus("Worker initialized");

      if (!workerApi.typedWasmSudoku) {
        throw new Error("Race condition while initializing wasm sudoku worker");
      }

      const wasmSudokuProxy = workerApi.typedWasmSudoku as unknown as Comlink.Remote<TypedWasmSudoku>;
      setWasmSudokuProxy(wasmSudokuProxy);

      const transportSudoku = await wasmSudokuProxy.getSudoku();
      setSudoku(transportSudoku);
    })().catch(err => setLoadingStatus(`Unexpected error: ${err}`))
  }, []);

  let content;

  if (!sudoku || !wasmSudokuProxy) {
    content = <><CircularProgress/>{loadingStatus}</>
  } else {
    content = <Sudoku sudoku={sudoku} setSudoku={setSudoku} wasmSudokuProxy={wasmSudokuProxy}/>
  }

  return <MyTheme>
    {content}
  </MyTheme>
};
