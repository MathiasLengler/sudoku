import { Snapshot, useRecoilCallback } from "recoil";
import type { WorldPosition } from "../../types";
import { remoteWasmCellWorldState, type RemoteWasmCellWorld } from "../state/worker";

async function getRemoteWasmCellWorld(snapshot: Snapshot): Promise<RemoteWasmCellWorld> {
    return await snapshot.getPromise(remoteWasmCellWorldState);
}

// TODO: set/get grid at grid index
//  set grid when opening world
//  get grid when selecting grid

// TODO: port changeGrid
// #[wasm_bindgen(js_name = changeGrid)]
// pub fn change_grid(&mut self, _dir: IRelativeGridDir) -> Result<()> {
//
// let dir = import_dir(dir)?;
//
// let new_grid_index =
//     self.grid_index
//         .adjacent(dir, self.world.grid_dim())
//         .ok_or(anyhow!(
//             "Currently at world boundary {:?}, can't move {:?}",
//             self.grid_index,
//             dir
//         ))?;
//
// let DynamicSudoku::Base3(sudoku_base_3) = &self.sudoku else {
//     panic!("POC: base 3 only")
// };
//
// self.world
//     .set_grid_at(sudoku_base_3.grid(), self.grid_index);
//
// self.sudoku =
//     DynamicSudoku::Base3(Sudoku::with_grid(self.world.to_grid_at(new_grid_index)));
// self.grid_index = new_grid_index;
//
// Ok(())

export function useSetWorldGridAsSingle() {
    return useRecoilCallback(
        ({ snapshot, set: _set }) =>
            async (world_index_grid: WorldPosition) => {
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
