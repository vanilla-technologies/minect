pub mod geometry3;
pub mod structure;

use geometry3::Coordinate3;
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::BufWriter,
};
use structure::{
    placement::{self, place_commands, CommandBlock},
    Block, StructureBuilder,
};

pub struct InjectionConnection {}

impl InjectionConnection {
    pub fn new() -> InjectionConnection {
        InjectionConnection {}
    }

    pub fn inject_group(&self, group: Vec<Command>) {
        let mut builder = StructureBuilder::new();
        let blocks = place_commands(group);
        for block in blocks {
            builder.add_block(block);
        }
        let structure = builder.build();

        let file = File::create("/mnt/c/Users/Adrian/AppData/Roaming/.minecraft/saves/Scribble/generated/minecraft/structures/foo.nbt").unwrap();
        let mut writer = BufWriter::new(file);
        nbt::to_gzip_writer(&mut writer, &structure, None).unwrap();
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
    use crate::{Command, InjectionConnection};

    #[test]
    fn it_works() {
        // given:
        let connection = InjectionConnection::new();
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
        connection.inject_group(group);

        // then:
    }
}
