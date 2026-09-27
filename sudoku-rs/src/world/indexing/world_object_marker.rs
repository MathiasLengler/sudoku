use std::fmt::Debug;
use std::hash::Hash;

// Brand key of `WorldPosition`/`WorldDim`: only its type is exported, it has no runtime value.
#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_WORLD_OBJECT_BRAND: &str =
    "declare const worldObject: unique symbol;\nexport type WorldObjectKey = typeof worldObject;";

#[cfg_attr(feature = "wasm", derive(tsify::Tsify), tsify(type = "\"cell\""))]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CellMarker;
impl WorldObject for CellMarker {}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify), tsify(type = "\"grid\""))]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GridMarker;
impl WorldObject for GridMarker {}

pub trait WorldObject
where
    Self: Ord + Hash + Clone + Copy + Debug + Default + Send + Sync + 'static,
{
}
