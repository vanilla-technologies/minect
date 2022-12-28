// Minect is library that allows a program to connect to a running Minecraft instance without
// requiring any Minecraft mods.
//
// © Copyright (C) 2021, 2022 Adrodoc <adrodoc55@googlemail.com> & skess42 <skagaros@gmail.com>
//
// This file is part of Minect.
//
// Minect is free software: you can redistribute it and/or modify it under the terms of the GNU
// General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// Minect is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
// the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
// Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Minect.
// If not, see <http://www.gnu.org/licenses/>.

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
