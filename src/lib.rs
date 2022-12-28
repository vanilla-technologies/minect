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

pub mod log_observer;

mod geometry3;
mod placement;
mod structure;
mod utils;

use crate::{
    geometry3::Coordinate3,
    structure::{new_command_block, new_structure_block, CommandBlockKind},
};
use flate2::{write::GzEncoder, Compression};
use fs3::FileExt;
use geometry3::Direction3;
use log_observer::{LogEvent, LogObserver};
use placement::{place_commands, CommandBlock};
use std::{
    collections::BTreeMap,
    fmt::{self, Display},
    fs::{create_dir_all, write, File, OpenOptions},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use structure::{Block, StructureBuilder};
use tokio::sync::mpsc::UnboundedReceiver;
use utils::io_invalid_data;

pub struct MinecraftConnection {
    identifier: String,
    structures_dir: PathBuf,
    datapack_dir: PathBuf,
    namespace: String,
    log_observer: LogObserver,
}

impl MinecraftConnection {
    pub fn new(identifier: String, world_dir: PathBuf, log_file: PathBuf) -> MinecraftConnection {
        let namespace = "minect".to_string();
        MinecraftConnection {
            structures_dir: world_dir
                .join("generated")
                .join(&namespace)
                .join("structures")
                .join(&identifier),
            datapack_dir: world_dir.join("datapacks/minect"),
            identifier,
            namespace,
            log_observer: LogObserver::new(log_file),
        }
    }

    pub fn add_general_listener(&mut self) -> UnboundedReceiver<LogEvent> {
        self.log_observer.add_general_listener()
    }

    pub fn add_listener(&mut self, listener: &str) -> UnboundedReceiver<LogEvent> {
        self.log_observer.add_listener(listener)
    }

    pub fn inject_commands(&self, commands: Vec<String>) -> io::Result<()> {
        if !self.datapack_dir.is_dir() {
            self.extract_datapack()?;
        }

        create_dir_all(&self.structures_dir)?;
        let id = self.get_structure_id()?;
        let next_id = id.wrapping_add(1);

        let mut builder = StructureBuilder::new();
        builder.add_block(new_structure_block(
            format!("{}:{}/{}", self.namespace, self.identifier, next_id),
            "LOAD".to_string(),
            Coordinate3(0, 0, 0),
        ));
        builder.add_block(Block {
            name: "minecraft:stone".to_string(),
            pos: Coordinate3(0, 1, 0),
            properties: BTreeMap::new(),
            nbt: None,
        });
        builder.add_block(Block {
            name: "minecraft:redstone_block".to_string(),
            pos: Coordinate3(0, 2, 0),
            properties: BTreeMap::new(),
            nbt: None,
        });
        builder.add_block(Block {
            name: "minecraft:activator_rail".to_string(),
            pos: Coordinate3(0, 3, 0),
            properties: BTreeMap::new(),
            nbt: None,
        });
        builder.add_block(new_command_block(
            CommandBlockKind::Repeat,
            None,
            "execute \
                positioned ~ ~-1 ~ \
                align xyz \
                unless entity @e[type=area_effect_cloud,dx=1,dy=1,dz=1,tag=minect_connection] \
                run summon area_effect_cloud ~.5 ~.5 ~.5 {\
                    Tags:[minect_connection],\
                    Age:-2147483648,\
                    Duration:-1,\
                    WaitTime:-2147483648,\
                    }"
            .to_string(),
            false,
            true,
            Direction3::Down,
            Coordinate3(0, 4, 0),
        ));

        const CMD_BLOCK_OFFSET: Coordinate3<i32> = Coordinate3(1, 0, 1);

        let cmd_blocks = place_commands(commands);
        if let Some(mut block) = clean_up_cmd_block(&cmd_blocks) {
            block.pos += CMD_BLOCK_OFFSET;
            builder.add_block(block);
        }

        let mut first = true;
        for mut block in cmd_blocks.into_iter().map(Block::from) {
            if first {
                block.name = "minecraft:command_block".to_string();
                first = false;
            }
            block.pos += CMD_BLOCK_OFFSET;
            builder.add_block(block);
        }

        let structure = builder.build();

        // Create a corrupt file that prevents Minecraft from caching it
        let next_structure_file =
            File::create(self.structures_dir.join(format!("{}.nbt", next_id)))?;
        GzEncoder::new(next_structure_file, Compression::none()).write_all(&[u8::MAX, 0, 0])?;

        let temporary_file = self.structures_dir.join("tmp.nbt");
        let file = File::create(&temporary_file)?;
        let mut writer = BufWriter::new(file);
        nbt::to_gzip_writer(&mut writer, &structure, None).unwrap();

        let structure_file = self.structures_dir.join(format!("{}.nbt", id));
        // Create file as atomically as possible
        std::fs::rename(&temporary_file, &structure_file)?;

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

    fn extract_datapack(&self) -> io::Result<()> {
        macro_rules! include_datapack_template {
            ($relative_path:expr) => {
                include_str!(concat!(env!("OUT_DIR"), "/src/datapack/", $relative_path))
            };
        }
        macro_rules! extract_datapack_file {
            ($relative_path:expr) => {
                create_file(
                    self.datapack_dir.join($relative_path),
                    include_datapack_template!($relative_path),
                )
            };
        }

        extract_datapack_file!("data/minecraft/tags/functions/load.json")?;
        extract_datapack_file!("data/minecraft/tags/functions/tick.json")?;
        extract_datapack_file!("data/minect/functions/enable_logging.mcfunction")?;
        extract_datapack_file!("data/minect/functions/install.mcfunction")?;
        extract_datapack_file!("data/minect/functions/load.mcfunction")?;
        extract_datapack_file!("data/minect/functions/pulse_redstone.mcfunction")?;
        extract_datapack_file!("data/minect/functions/reload.mcfunction")?;
        extract_datapack_file!("data/minect/functions/reset_logging.mcfunction")?;
        extract_datapack_file!("data/minect/functions/tick.mcfunction")?;
        extract_datapack_file!("data/minect/functions/uninstall.mcfunction")?;
        extract_datapack_file!("pack.mcmeta")?;

        Ok(())
    }
}

fn clean_up_cmd_block(cmd_blocks: &[CommandBlock<String>]) -> Option<Block> {
    let (last_pos, last_dir) = cmd_blocks
        .last()
        .map(|cmd_block| (cmd_block.coordinate, cmd_block.direction))?;

    let mut pos = last_pos;
    pos[last_dir.axis()] += last_dir.signum() as i32;

    let max_pos = cmd_blocks
        .iter()
        .fold(pos, |pos, block| Coordinate3::max(pos, block.coordinate));

    let relative_min = -pos;
    let relative_max = relative_min + max_pos;
    let fill_cmd = format!(
        "fill ~{} ~{} ~{} ~{} ~{} ~{} air",
        relative_min.0,
        relative_min.1,
        relative_min.2,
        relative_max.0,
        relative_max.1,
        relative_max.2
    );
    Some(new_command_block(
        CommandBlockKind::Chain,
        None,
        fill_cmd,
        false,
        true,
        -last_dir,
        pos,
    ))
}

fn create_file(path: impl AsRef<Path>, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        create_dir_all(parent)?;
    }
    write(path, contents)
}

pub struct MinecraftConnectionBuilder {
    identifier: String,
    world_dir: PathBuf,
    log_file: Option<PathBuf>,
}

impl MinecraftConnectionBuilder {
    pub fn from_ref(identifier: &str, world_dir: impl AsRef<Path>) -> MinecraftConnectionBuilder {
        MinecraftConnectionBuilder::new(identifier.to_string(), world_dir.as_ref().to_path_buf())
    }

    pub fn new(identifier: String, world_dir: PathBuf) -> MinecraftConnectionBuilder {
        MinecraftConnectionBuilder {
            identifier,
            world_dir,
            log_file: None,
        }
    }

    pub fn log_file(mut self, log_file: PathBuf) -> MinecraftConnectionBuilder {
        self.log_file = Some(log_file);
        self
    }

    pub fn build(self) -> MinecraftConnection {
        let world_dir = self.world_dir;
        let log_file = self
            .log_file
            .unwrap_or_else(|| log_file_from_world_dir(&world_dir));
        MinecraftConnection::new(self.identifier, world_dir, log_file)
    }
}

fn log_file_from_world_dir(world_dir: &PathBuf) -> PathBuf {
    let panic_invalid_dir = || {
        panic!(
            "Expected world_dir to be in .minecraft/saves, but was: {}",
            world_dir.display()
        )
    };
    let minecraft_dir = world_dir
        .parent()
        .unwrap_or_else(panic_invalid_dir)
        .parent()
        .unwrap_or_else(panic_invalid_dir);
    minecraft_dir.join("logs/latest.log")
}

impl From<CommandBlock<String>> for Block {
    fn from(cmd_block: CommandBlock<String>) -> Self {
        new_command_block(
            CommandBlockKind::Chain,
            None,
            cmd_block.command.unwrap_or_default(),
            false,
            true,
            cmd_block.direction,
            cmd_block.coordinate,
        )
    }
}

pub struct LoggedCommand {
    custom_name: Option<String>,
    command: String,
}

impl LoggedCommand {
    pub fn from_str(command: &str) -> LoggedCommand {
        LoggedCommand::from(command.to_string())
    }
}

impl From<String> for LoggedCommand {
    fn from(command: String) -> Self {
        LoggedCommand {
            command,
            custom_name: None,
        }
    }
}

impl Display for LoggedCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "execute at @e[type=area_effect_cloud,tag=minect_connection,limit=1] \
                run summon command_block_minecart ~ ~ ~ {",
        )?;
        if let Some(custom_name) = self.custom_name.as_ref() {
            write!(f, "\"CustomName\":\"{}\",", escape_json(custom_name))?;
        }
        write!(f, "\"Command\":\"{}\",", self.command)?;
        f.write_str(
            "\
            \"Tags\":[\"minect_impulse\"],\
            \"LastExecution\":1L,\
            \"TrackOutput\":false,\
        }",
        )
    }
}

fn escape_json(json: &str) -> String {
    json.replace("\\", "\\\\").replace('"', "\\\"")
}

impl placement::Command for LoggedCommand {
    fn is_conditional(&self) -> bool {
        false
    }
}

impl LoggedCommand {
    pub fn builder(command: String) -> LoggedCommandBuilder {
        LoggedCommandBuilder::new(command)
    }
}

pub struct LoggedCommandBuilder {
    custom_name: Option<String>,
    command: String,
}

impl LoggedCommandBuilder {
    pub fn new(command: String) -> LoggedCommandBuilder {
        LoggedCommandBuilder {
            custom_name: None,
            command,
        }
    }

    pub fn custom_name(mut self, custom_name: String) -> LoggedCommandBuilder {
        self.custom_name = Some(custom_name);
        self
    }

    pub fn name(self, name: &str) -> LoggedCommandBuilder {
        self.custom_name(format!(r#"{{"text":"{}"}}"#, escape_json(name)))
    }

    pub fn build(self) -> LoggedCommand {
        LoggedCommand {
            custom_name: self.custom_name,
            command: self.command,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;
    use tokio::time::timeout;

    const TEST_WORLD_DIR: &str = env!("TEST_WORLD_DIR");

    #[tokio::test]
    #[serial]
    async fn test_tag() -> io::Result<()> {
        // given:
        let mut connection = MinecraftConnectionBuilder::from_ref("test", TEST_WORLD_DIR).build();
        let name = "test";
        let commands = vec![
            "say running test_tag".to_string(),
            LoggedCommand::from_str("function minect:enable_logging").to_string(),
            LoggedCommand::builder("tag @s add success".to_string())
                .name(name)
                .build()
                .to_string(),
            LoggedCommand::from_str("function minect:reset_logging").to_string(),
        ];
        let mut events = connection.add_listener(name);

        // when:
        connection.inject_commands(commands)?;

        // then:
        let event = timeout(Duration::from_secs(5), events.recv())
            .await?
            .unwrap();
        assert_eq!(event.message, format!("Added tag 'success' to {}", name));

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_score_objective() -> io::Result<()> {
        // given:
        let mut connection = MinecraftConnectionBuilder::from_ref("test", TEST_WORLD_DIR).build();
        let name = "test";
        let commands = vec![
            "say running test_score_objective".to_string(),
            LoggedCommand::from_str("function minect:enable_logging").to_string(),
            LoggedCommand::builder("scoreboard objectives add success dummy".to_string())
                .name(name)
                .build()
                .to_string(),
            LoggedCommand::from_str("scoreboard objectives remove success").to_string(),
            LoggedCommand::from_str("function minect:reset_logging").to_string(),
        ];
        let mut events = connection.add_listener(name);

        // when:
        connection.inject_commands(commands)?;

        // then:
        let event = timeout(Duration::from_secs(5), events.recv())
            .await?
            .unwrap();
        assert_eq!(event.message, "Created new objective [success]");

        Ok(())
    }
}
