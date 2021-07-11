mod nbt;

use crate::{
    geometry3::Coordinate3,
    structure::nbt::{PaletteBlock, Structure, StructureBlock},
};
use std::collections::BTreeMap;

pub struct StructureBuilder {
    size: Coordinate3<i32>,
    palette: Vec<PaletteBlock>,
    blocks: Vec<StructureBlock>,
}

impl StructureBuilder {
    pub fn new() -> StructureBuilder {
        StructureBuilder {
            size: Coordinate3(0, 0, 0),
            palette: Vec::new(),
            blocks: Vec::new(),
        }
    }

    pub fn add_block(&mut self, block: Block) {
        let Block {
            name,
            pos,
            properties,
            nbt,
        } = block;
        let palette_block = PaletteBlock { name, properties };
        let index = if let Some(index) = self.palette.iter().position(|it| *it == palette_block) {
            index
        } else {
            self.palette.push(palette_block);
            self.palette.len() - 1
        };
        let block = StructureBlock {
            state: index as i32,
            pos: pos.into(),
            nbt,
        };
        self.blocks.push(block);
        self.size = Coordinate3::max(self.size, pos + Coordinate3(1, 1, 1));
    }

    pub fn build(self) -> Structure {
        Structure {
            data_version: 2724,
            size: self.size.into(),
            palette: self.palette,
            blocks: self.blocks,
            entities: Vec::new(),
        }
    }
}

pub struct Block {
    pub name: String,
    pub pos: Coordinate3<i32>,
    pub properties: BTreeMap<String, String>,
    pub nbt: Option<::nbt::Value>,
}
