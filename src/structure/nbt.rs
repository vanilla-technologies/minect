use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Structure {
    #[serde(rename = "DataVersion")]
    pub(crate) data_version: i32,
    pub(crate) size: Vec<i32>,
    // size: [i32; 3],
    pub(crate) palette: Vec<PaletteBlock>,
    pub(crate) blocks: Vec<StructureBlock>,
    pub(crate) entities: Vec<StructureEntity>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct PaletteBlock {
    #[serde(rename = "Name")]
    pub(crate) name: String,
    #[serde(rename = "Properties", default)]
    pub(crate) properties: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct StructureBlock {
    pub(crate) state: i32,
    // state: usize,
    pub(crate) pos: Vec<i32>,
    // pos: [i32; 3],
    pub(crate) nbt: Option<nbt::Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct StructureEntity {
    pub(crate) pos: Vec<f64>,
    // pos: [f64; 3],
    pub(crate) block_pos: Vec<i32>,
    // block_pos: [i32; 3],
    pub(crate) nbt: nbt::Value,
}
