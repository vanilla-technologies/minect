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

mod geometry3;
mod json;
pub mod log;
mod placement;
mod structure;
mod utils;

use crate::{
    log::{
        enable_logging_command, observer::LogObserver, reset_logging_command,
        summon_named_entity_command, LogEvent, SummonNamedEntityOutput,
    },
    placement::generate_structure,
    utils::io_invalid_data,
};
use ::log::error;
use flate2::{write::GzEncoder, Compression};
use fs3::FileExt;
use std::{
    fmt::Display,
    fs::{create_dir_all, remove_dir_all, remove_file, write, File, OpenOptions},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use tokio_stream::Stream;

pub struct MinecraftConnection {
    identifier: String,
    structures_dir: PathBuf,
    datapack_dir: PathBuf,
    log_file: PathBuf,
    log_observer: Option<LogObserver>,
    loaded_listener_initialized: bool,
    _private: (),
}

const NAMESPACE: &str = "minect";

impl MinecraftConnection {
    pub fn builder(
        identifier: impl Into<String>,
        world_dir: impl Into<PathBuf>,
    ) -> MinecraftConnectionBuilder {
        MinecraftConnectionBuilder::new(identifier, world_dir)
    }

    fn new(identifier: String, world_dir: PathBuf, log_file: PathBuf) -> MinecraftConnection {
        MinecraftConnection {
            structures_dir: world_dir
                .join("generated")
                .join(NAMESPACE)
                .join("structures")
                .join(&identifier),
            datapack_dir: world_dir.join("datapacks").join(NAMESPACE),
            identifier,
            log_file,
            log_observer: None,
            loaded_listener_initialized: false,
            _private: (),
        }
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }

    /// The root directory of the datapack used to operate the connection in Minecraft.
    pub fn get_datapack_dir(&self) -> &Path {
        &self.datapack_dir
    }

    pub fn add_listener(&mut self) -> impl Stream<Item = LogEvent> {
        self.get_log_observer().add_listener()
    }

    pub fn add_named_listener(&mut self, name: impl Into<String>) -> impl Stream<Item = LogEvent> {
        self.get_log_observer().add_named_listener(name)
    }

    fn get_log_observer(&mut self) -> &mut LogObserver {
        if self.log_observer.is_none() {
            // Start LogObserver only when needed
            self.log_observer = Some(LogObserver::new(&self.log_file));
        }
        self.log_observer.as_mut().unwrap() // Unwrap is safe because we just assigned the value
    }

    pub fn inject_commands(
        &mut self,
        commands: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl ToString>>,
    ) -> io::Result<()> {
        if !self.datapack_dir.is_dir() {
            // Create datapack only when needed
            self.create_datapack()?;
        }
        if !self.loaded_listener_initialized {
            self.init_loaded_listener();
        }
        create_dir_all(&self.structures_dir)?;

        let id = self.increment_and_get_structure_id()?;
        let next_id = id.wrapping_add(1);

        let (commands, commands_len) = prepend_loaded_command(id, commands);
        let structure = generate_structure(&self.identifier, next_id, commands, commands_len);

        // Create a corrupt file that prevents Minecraft from caching it
        let next_structure_file = File::create(self.get_structure_file(next_id))?;
        GzEncoder::new(next_structure_file, Compression::none()).write_all(&[u8::MAX, 0, 0])?;

        let temporary_file = self.get_structure_file("tmp");
        let file = File::create(&temporary_file)?;
        let mut writer = BufWriter::new(file);
        nbt::to_gzip_writer(&mut writer, &structure, None).unwrap();

        let structure_file = self.get_structure_file(id);
        // Create file as atomically as possible
        std::fs::rename(&temporary_file, &structure_file)?;

        Ok(())
    }

    fn init_loaded_listener(&mut self) {
        let structures_dir = self.structures_dir.clone();
        let listener = LoadedListener { structures_dir };
        self.get_log_observer().add_loaded_listener(listener);
        self.loaded_listener_initialized = true;
    }

    fn increment_and_get_structure_id(&self) -> Result<u64, io::Error> {
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

    fn get_structure_file(&self, id: impl Display) -> PathBuf {
        self.structures_dir.join(format!("{}.nbt", id))
    }

    /// Creates the datapack used to operate the connection in Minecraft at the directory
    /// [Self::get_datapack_dir()].
    pub fn create_datapack(&self) -> io::Result<()> {
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

    /// Removes the datapack used to operate the connection in Minecraft at the directory
    /// [Self::get_datapack_dir()].
    pub fn remove_datapack(&self) -> io::Result<()> {
        remove_dir_all(&self.datapack_dir)
    }
}

struct LoadedListener {
    structures_dir: PathBuf,
}
impl LoadedListener {
    fn on_event(&self, event: LogEvent) {
        if let Some(id) = parse_loaded_output(&event.output) {
            let structure_file = self.get_structure_file(id);
            if let Err(error) = remove_file(&structure_file) {
                error!(
                    "Failed to remove structure file {} due to: {}",
                    structure_file.display(),
                    error
                );
            }
            // Remove all previous structure files in case they are still there
            // (for instance because a structure was loaded while no connection was active)
            let mut i = 1;
            while let Ok(()) = remove_file(self.get_structure_file(id.wrapping_sub(i))) {
                i += 1;
            }
        }
    }

    fn get_structure_file(&self, id: impl Display) -> PathBuf {
        self.structures_dir.join(format!("{}.nbt", id))
    }
}

const STRUCTURE_LOADED_OUTPUT_PREFIX: &str = "minect_loaded_";

fn parse_loaded_output(output: &str) -> Option<u64> {
    let output = output.parse::<SummonNamedEntityOutput>().ok()?;
    let id = &output.name.strip_prefix(STRUCTURE_LOADED_OUTPUT_PREFIX)?;
    id.parse().ok()
}

fn prepend_loaded_command(
    id: u64,
    commands: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl ToString>>,
) -> (impl Iterator<Item = String>, usize) {
    let loaded_cmds = [
        enable_logging_command(),
        summon_named_entity_command(&format!("{}{}", STRUCTURE_LOADED_OUTPUT_PREFIX, id)),
        reset_logging_command(),
    ];
    let commands = commands.into_iter();
    let commands_len = loaded_cmds.len() + commands.len();
    let commands = loaded_cmds
        .into_iter()
        .chain(commands.map(|it| it.to_string()));
    (commands, commands_len)
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
    fn new(
        identifier: impl Into<String>,
        world_dir: impl Into<PathBuf>,
    ) -> MinecraftConnectionBuilder {
        MinecraftConnectionBuilder {
            identifier: identifier.into(),
            world_dir: world_dir.into(),
            log_file: None,
        }
    }

    pub fn log_file(mut self, log_file: impl Into<PathBuf>) -> MinecraftConnectionBuilder {
        self.log_file = Some(log_file.into());
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
