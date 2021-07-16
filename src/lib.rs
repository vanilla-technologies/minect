mod geometry3;
mod log_observer;
mod placement;
mod structure;
mod utils;

use fs3::FileExt;
use geometry3::Direction3;
use log_observer::{LogEvent, LogObserver};
use placement::{place_commands, CommandBlock};
use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use structure::{Block, StructureBuilder};
use tokio::sync::mpsc::UnboundedReceiver;
use utils::io_invalid_data;

use crate::{
    geometry3::Coordinate3,
    structure::{new_command_block, new_structure_block, CommandBlockKind},
};

pub struct InjectionConnection {
    structures_dir: PathBuf,
    namespace: String,
    identifier: String,
    log_observer: LogObserver,
}

impl InjectionConnection {
    pub fn new<W: AsRef<Path>, L: AsRef<Path>>(
        identifier: &str,
        world_dir: W,
        log_file: L,
    ) -> InjectionConnection {
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
            log_observer: LogObserver::new(log_file),
        }
    }

    pub fn add_listener(&mut self, listener: &str) -> UnboundedReceiver<LogEvent> {
        self.log_observer.add_listener(listener)
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
            None,
            "setblock ~ ~-1 ~ redstone_block".to_string(),
            false,
            true,
            Direction3::Up,
            Coordinate3(0, 2, 0),
        ));
        builder.add_block(new_command_block(
            CommandBlockKind::Chain,
            None,
            "setblock ~ ~-2 ~ stone".to_string(),
            false,
            true,
            Direction3::Up,
            Coordinate3(0, 3, 0),
        ));
        builder.add_block(new_command_block(
            CommandBlockKind::Chain,
            None,
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

pub struct Command {
    name: Option<String>,
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

        let (name, command) = if let Some(command) = cmd_block.command {
            (command.name, command.command)
        } else {
            (None, String::default())
        };

        new_command_block(
            CommandBlockKind::Chain,
            name,
            command,
            conditional,
            true,
            cmd_block.direction,
            cmd_block.coordinate,
        )
    }
}

pub struct CommandBuilder {
    custom_name: Option<String>,
    command: String,
    conditional: bool,
}

impl CommandBuilder {
    pub fn new(command: &str) -> CommandBuilder {
        CommandBuilder {
            custom_name: None,
            command: command.to_string(),
            conditional: false,
        }
    }

    pub fn custom_name(mut self, custom_name: Option<String>) -> CommandBuilder {
        self.custom_name = custom_name;
        self
    }

    pub fn name(self, name: Option<&str>) -> CommandBuilder {
        self.custom_name(name.map(|name| format!(r#"{{"text":"{}"}}"#, name)))
    }

    pub fn conditional(mut self, conditional: bool) -> CommandBuilder {
        self.conditional = conditional;
        self
    }

    pub fn build(self) -> Command {
        Command {
            name: self.custom_name,
            command: self.command,
            conditional: self.conditional,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> io::Result<()> {
        // given:
        let minecraft_dir = Path::new("/mnt/c/Users/Adrodoc/AppData/Roaming/.minecraft");
        let world_dir = minecraft_dir.join("saves/Scribble");
        let log_file = minecraft_dir.join("logs/latest.log");
        let mut connection = InjectionConnection::new("foo", world_dir, log_file);

        let name = "test1";

        let group = vec![
            CommandBuilder::new("say teleporting").build(),
            CommandBuilder::new("function abc:123").build(),
            CommandBuilder::new("execute at @p run teleport @p ~ ~1 ~")
                .name(Some(name))
                .build(),
        ];

        // when:
        connection.inject_group(group)?;
        let mut listener = connection.add_listener(name);

        let event = listener.recv().await;

        println!("{:?}", event);

        // then:
        Ok(())
    }
}
