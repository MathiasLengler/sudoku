use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::cell::dynamic::DynamicCell;

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicGrid {
    cells: Vec<DynamicCell>,
}
