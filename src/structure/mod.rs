mod nbt;

use crate::{
    geometry3::{Coordinate3, Direction3},
    structure::nbt::{PaletteBlock, Structure, StructureBlock},
};
use ::nbt::Value;
use std::{
    collections::{BTreeMap, HashMap},
    iter::FromIterator,
};

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
    pub nbt: Option<Value>,
}

pub fn new_structure_block(name: String, mode: String, pos: Coordinate3<i32>) -> Block {
    Block {
        name: "minecraft:structure_block".to_string(),
        pos,
        properties: BTreeMap::new(),
        nbt: Some(Value::Compound(HashMap::from_iter([
            ("name".to_string(), Value::String(name)),
            ("mode".to_string(), Value::String(mode)),
        ]))),
    }
}

pub fn new_command_block(
    kind: CommandBlockKind,
    name: Option<String>,
    command: String,
    conditional: bool,
    always_active: bool,
    facing: Direction3,
    pos: Coordinate3<i32>,
) -> Block {
    let mut properties = BTreeMap::from_iter([("facing".to_string(), facing.to_string())]);
    if conditional {
        properties.insert("conditional".to_string(), "true".to_string());
    }

    let mut nbt = HashMap::from_iter([("Command".to_string(), Value::String(command))]);
    if let Some(name) = name {
        nbt.insert("CustomName".to_string(), Value::String(name));
    }
    if always_active {
        nbt.insert("auto".to_string(), Value::Byte(1));
    }

    Block {
        name: kind.block_name().to_string(),
        pos,
        properties,
        nbt: Some(Value::Compound(nbt)),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandBlockKind {
    Impulse,
    Chain,
    Repeat,
}

impl CommandBlockKind {
    fn block_name(&self) -> &'static str {
        match self {
            CommandBlockKind::Impulse => "minecraft:command_block",
            CommandBlockKind::Chain => "minecraft:chain_command_block",
            CommandBlockKind::Repeat => "minecraft:repeating_command_block",
        }
    }
}
