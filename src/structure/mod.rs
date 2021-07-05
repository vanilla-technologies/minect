pub mod nbt;
pub mod placement;

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

    pub fn add_block<B: Block>(&mut self, block: B) {
        let palette_block = PaletteBlock {
            name: block.name(),
            properties: block.properties(),
        };
        let index = if let Some(index) = self.palette.iter().position(|it| *it == palette_block) {
            index
        } else {
            self.palette.push(palette_block);
            self.palette.len() - 1
        };
        let pos = block.pos();
        let block = StructureBlock {
            state: index as i32,
            pos: pos.into(),
            nbt: block.nbt(),
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

pub trait Block {
    fn name(&self) -> String;

    fn pos(&self) -> Coordinate3<i32>;

    fn properties(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    fn nbt(&self) -> Option<::nbt::Value> {
        None
    }
}
