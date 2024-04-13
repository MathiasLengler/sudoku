import { Snapshot, useRecoilCallback } from "recoil";
import { type RemoteWasmCellWorld } from "../state/worker";
import { remoteWasmCellWorldState, type WorldGridPosition } from "../state/world";

async function getRemoteWasmCellWorld(snapshot: Snapshot): Promise<RemoteWasmCellWorld> {
    return await snapshot.getPromise(remoteWasmCellWorldState);
}

// TODO: set/get grid at grid index
//  set grid when opening world
//  get grid when selecting grid

// TODO: port changeGrid

export function useSetWorldGridAsSingle() {
    return useRecoilCallback(
        ({ snapshot, set: _set }) =>
            async (world_index_grid: WorldGridPosition) => {
                console.log("changeGrid", world_index_grid);
                const remoteWasmCellWorldProxy = await getRemoteWasmCellWorld(snapshot);
                console.log(remoteWasmCellWorldProxy);

                const newGrid = await remoteWasmCellWorldProxy.toGridAt(world_index_grid);

                console.log("newGrid", newGrid);
                // await wasmCellWorldProxy.???;
                // await updateSudoku({ set, wasmSudokuProxy });
            },
        [],
    );
}
