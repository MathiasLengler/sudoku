import { Snapshot, useRecoilCallback } from "recoil";
import type { TileIndex } from "../../types";
import { remoteWasmCellWorldState, type RemoteWasmCellWorld } from "../state/worker";

async function getRemoteWasmCellWorld(snapshot: Snapshot): Promise<RemoteWasmCellWorld> {
    return await snapshot.getPromise(remoteWasmCellWorldState);
}

// TODO: set/get grid at tile index
//  set grid when opening world
//  get grid when selecting tile

// TODO: port changeTile
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

export function useSetWorldTileAsSingle() {
    return useRecoilCallback(
        ({ snapshot, set: _set }) =>
            async (tileIndex: TileIndex) => {
                console.log("changeTile", tileIndex);
                const remoteWasmCellWorldProxy = await getRemoteWasmCellWorld(snapshot);
                console.log(remoteWasmCellWorldProxy);

                const newGrid = await remoteWasmCellWorldProxy.toGridAt(tileIndex);

                console.log("newGrid", newGrid);
                // await wasmCellWorldProxy.???;
                // await updateSudoku({ set, wasmSudokuProxy });
            },
        [],
    );
}
