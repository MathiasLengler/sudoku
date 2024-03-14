import { Snapshot, useRecoilCallback } from "recoil";
import type { RelativeTileDir } from "../../types";
import type { WasmCellWorldProxy } from "../../spawnWorker";
import { remoteWorkerApiState } from "../state/worker";

async function getWasmCellWorldProxy(snapshot: Snapshot): Promise<WasmCellWorldProxy> {
    const { wasmCellWorldProxy } = await snapshot.getPromise(remoteWorkerApiState);
    return wasmCellWorldProxy;
}

// #[wasm_bindgen(js_name = changeTile)]
// pub fn change_tile(&mut self, _dir: IRelativeTileDir) -> Result<()> {
//
// let dir = import_dir(dir)?;
//
// let new_tile_index =
//     self.tile_index
//         .adjacent(dir, self.world.tile_dim())
//         .ok_or(anyhow!(
//             "Currently at world boundary {:?}, can't move {:?}",
//             self.tile_index,
//             dir
//         ))?;
//
// let DynamicSudoku::Base3(sudoku_base_3) = &self.sudoku else {
//     panic!("POC: base 3 only")
// };
//
// self.world
//     .set_grid_at(sudoku_base_3.grid(), self.tile_index);
//
// self.sudoku =
//     DynamicSudoku::Base3(Sudoku::with_grid(self.world.to_grid_at(new_tile_index)));
// self.tile_index = new_tile_index;
//
// Ok(())

export function useChangeTile() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async (dir: RelativeTileDir) => {
                console.log("changeTile", dir);
                const wasmCellWorldProxy = await getWasmCellWorldProxy(snapshot);
                console.log(wasmCellWorldProxy);
                // await wasmCellWorldProxy.???;
                // await updateSudoku({ set, wasmSudokuProxy });
            },
        []
    );
}
