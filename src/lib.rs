mod geometry3;
mod logfile_watcher;
mod placement;
mod structure;
mod utils;

use fs3::FileExt;
use geometry3::Direction3;
use placement::{place_commands, CommandBlock};
use std::{
    collections::{BTreeMap, HashMap},
    fs::{create_dir_all, File, OpenOptions},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    iter::FromIterator,
    path::{Path, PathBuf},
};
use structure::{Block, StructureBuilder};
use utils::io_invalid_data;

use crate::geometry3::Coordinate3;

pub struct InjectionConnection {
    structures_dir: PathBuf,
    namespace: String,
    identifier: String,
}

impl InjectionConnection {
    pub fn new<P: AsRef<Path>>(identifier: &str, world_dir: P) -> InjectionConnection {
        let namespace = "inject".to_string();
        InjectionConnection {
            structures_dir: world_dir
                .as_ref()
                .join("generated")
                .join(&namespace)
                .join("structures")
                .join(identifier),
            namespace,
            identifier: identifier.to_string(),
        }
    }

    pub fn inject_group(&self, group: Vec<Command>) -> io::Result<()> {
        create_dir_all(&self.structures_dir)?;
        let id = self.get_structure_id()?;

        let mut builder = StructureBuilder::new();
        builder.add_block(new_structure_block(
            format!(
                "{}:{}/{}",
                self.namespace,
                self.identifier,
                id.wrapping_add(1)
            ),
            "LOAD".to_string(),
            Coordinate3(0, 0, 0),
        ));
        builder.add_block(new_command_block(
            CommandBlockKind::Repeat,
            "setblock ~ ~-1 ~ redstone_block".to_string(),
            false,
            true,
            Direction3::Up,
            Coordinate3(0, 2, 0),
        ));
        builder.add_block(new_command_block(
            CommandBlockKind::Chain,
            "setblock ~ ~-2 ~ stone".to_string(),
            false,
            true,
            Direction3::Up,
            Coordinate3(0, 3, 0),
        ));
        builder.add_block(new_command_block(
            CommandBlockKind::Chain,
            "reload".to_string(),
            false,
            true,
            Direction3::Down,
            Coordinate3(0, 4, 0),
        ));
        let mut first = true;
        for mut block in place_commands(group).into_iter().map(Block::from) {
            if first {
                block.name = "minecraft:command_block".to_string();
                first = false;
            }
            block.pos += Coordinate3(1, 1, 1);
            builder.add_block(block);
        }
        let structure = builder.build();

        let structure_file = self.structures_dir.join(format!("{}.nbt", id));

        let file = File::create(structure_file)?;
        let mut writer = BufWriter::new(file);
        nbt::to_gzip_writer(&mut writer, &structure, None).unwrap();

        Ok(())
    }

    fn get_structure_id(&self) -> Result<u64, io::Error> {
        let path = self.structures_dir.join("id.txt");
        let mut id_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        id_file.lock_exclusive()?; // Automatically unlocked by dropping id_file at the end of this function

        let mut content = String::new();
        id_file.read_to_string(&mut content)?;

        let id = if content.is_empty() {
            0
        } else {
            content
                .parse::<u64>()
                .map_err(io_invalid_data)?
                .wrapping_add(1)
        };

        id_file.set_len(0)?;
        id_file.seek(SeekFrom::Start(0))?;
        id_file.write_all(id.to_string().as_bytes())?;
        Ok(id)
    }
}

fn new_structure_block(name: String, mode: String, pos: Coordinate3<i32>) -> Block {
    Block {
        name: "minecraft:structure_block".to_string(),
        pos,
        properties: BTreeMap::new(),
        nbt: Some(nbt::Value::Compound(HashMap::from_iter([
            ("name".to_string(), nbt::Value::String(name)),
            ("mode".to_string(), nbt::Value::String(mode)),
        ]))),
    }
}

fn new_command_block(
    kind: CommandBlockKind,
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

    let mut nbt = HashMap::from_iter([("Command".to_string(), nbt::Value::String(command))]);
    if always_active {
        nbt.insert("auto".to_string(), nbt::Value::Byte(1));
    }

    Block {
        name: kind.block_name().to_string(),
        pos,
        properties,
        nbt: Some(nbt::Value::Compound(nbt)),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommandBlockKind {
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

pub struct Command {
    command: String,
    conditional: bool,
}

impl placement::Command for Command {
    fn is_conditional(&self) -> bool {
        self.conditional
    }
}

impl From<CommandBlock<Command>> for Block {
    fn from(cmd_block: CommandBlock<Command>) -> Self {
        let conditional = cmd_block
            .command
            .as_ref()
            .map(|it| it.conditional)
            .unwrap_or_default();
        let command = cmd_block.command.map(|it| it.command).unwrap_or_default();
        new_command_block(
            CommandBlockKind::Chain,
            command,
            conditional,
            true,
            cmd_block.direction,
            cmd_block.coordinate,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> io::Result<()> {
        // given:
        let connection = InjectionConnection::new(
            "foo",
            "/mnt/c/Users/Adrian/AppData/Roaming/.minecraft/saves/Scribble",
        );
        let group = vec![
            Command {
                command: "say foo".to_string(),
                conditional: false,
            },
            Command {
                command: "say bar".to_string(),
                conditional: false,
            },
        ];

        // when:
        connection.inject_group(group)?;

        // then:
        Ok(())
    }
}
