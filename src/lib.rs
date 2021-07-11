pub mod geometry3;
pub mod logfile_watcher;
pub mod structure;

use fs3::FileExt;
use geometry3::Coordinate3;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fs::{create_dir_all, File, OpenOptions},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use structure::{
    placement::{self, place_commands, CommandBlock},
    Block, StructureBuilder,
};

pub struct InjectionConnection {
    structures_dir: PathBuf,
}

impl InjectionConnection {
    pub fn new<P: AsRef<Path>>(identifier: &str, world_dir: P) -> InjectionConnection {
        InjectionConnection {
            structures_dir: world_dir
                .as_ref()
                .join("generated/inject/structures")
                .join(identifier),
        }
    }

    pub fn inject_group(&self, group: Vec<Command>) -> io::Result<()> {
        let mut builder = StructureBuilder::new();
        let blocks = place_commands(group);
        for block in blocks {
            builder.add_block(block);
        }
        let structure = builder.build();

        create_dir_all(&self.structures_dir)?;
        let id = self.get_structure_id()?;
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
                .map_err(invalid_data)?
                .wrapping_add(1)
        };

        id_file.set_len(0)?;
        id_file.seek(SeekFrom::Start(0))?;
        id_file.write_all(id.to_string().as_bytes())?;
        Ok(id)
    }
}

fn invalid_data<E>(error: E) -> io::Error
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::InvalidData, error)
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

impl Block for CommandBlock<Command> {
    fn name(&self) -> String {
        "minecraft:command_block".to_string()
    }

    fn pos(&self) -> Coordinate3<i32> {
        self.coordinate
    }

    fn properties(&self) -> BTreeMap<String, String> {
        let mut properties = BTreeMap::new();
        if let Some(command) = &self.command {
            if command.conditional {
                properties.insert("conditional".to_string(), "true".to_string());
            }
        }
        properties.insert("facing".to_string(), self.direction.to_string());
        properties
    }

    fn nbt(&self) -> Option<nbt::Value> {
        let mut nbt = HashMap::new();
        if let Some(command) = &self.command {
            nbt.insert(
                "Command".to_string(),
                nbt::Value::String(command.command.clone()),
            );
        }
        Some(nbt::Value::Compound(nbt))
        // {auto: 0b, powered: 0b, LastExecution: 2341237L, SuccessCount: 1, UpdateLastExecution: 1b, conditionMet: 1b, CustomName: '{"text":"@"}', Command: "function debug:example/main/start", x: -6, y: 56, z: 14, id: "minecraft:command_block", LastOutput: '{"extra":[{"translate":"commands.function.success.single","with":["2","debug:example/main/start"]}],"text":"[16:38:42] "}', TrackOutput: 1b}
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
                command: "execute".to_string(),
                conditional: false,
            },
            Command {
                command: "bli".to_string(),
                conditional: true,
            },
            Command {
                command: "bla".to_string(),
                conditional: false,
            },
            Command {
                command: "blub".to_string(),
                conditional: true,
            },
        ];

        // when:
        connection.inject_group(group)?;

        // then:
        Ok(())
    }
}
