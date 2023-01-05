// Minect is library that allows a program to connect to a running Minecraft instance without
// requiring any Minecraft mods.
//
// © Copyright (C) 2021-2023 Adrodoc <adrodoc55@googlemail.com> & skess42 <skagaros@gmail.com>
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

#[macro_use]
mod macros;

mod connect;
mod geometry3;
mod io;
mod json;
pub mod log;
mod on_drop;
mod placement;
mod structure;
mod utils;

pub use crate::connect::ConnectError;

use crate::{
    connect::connect,
    io::{
        create, create_dir_all, io_error, remove_dir_all, remove_file, rename, write, IoErrorAtPath,
    },
    log::{
        enable_logging_command, observer::LogObserver, reset_logging_command,
        summon_named_entity_command, LogEvent, SummonNamedEntityOutput,
    },
    placement::generate_structure,
    structure::nbt::Structure,
    utils::io_invalid_data,
};
use ::log::error;
use fs3::FileExt;
use json::create_json_text_component;
use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use tokio_stream::Stream;

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

macro_rules! extract_datapack_file {
    ($output_path:expr, $relative_path:expr) => {{
        let path = $output_path.join($relative_path);
        let contents = include_datapack_template!($relative_path);
        write(&path, &contents)
    }};
}

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

    pub async fn connect(&mut self) -> Result<(), ConnectError> {
        connect(self).await
    }

    /// Creates the Minect datapack at directory [Self::get_datapack_dir()].
    pub fn create_datapack(&self) -> Result<(), IoErrorAtPath> {
        macro_rules! extract {
            ($relative_path:expr) => {
                extract_datapack_file!(self.datapack_dir, $relative_path)
            };
        }

        extract!("data/minecraft/tags/functions/load.json")?;
        extract!("data/minecraft/tags/functions/tick.json")?;
        extract!("data/minect_internal/functions/connect/align_to_chunk.mcfunction")?;
        extract!("data/minect_internal/functions/connect/remove_connector.mcfunction")?;
        extract!("data/minect_internal/functions/install.mcfunction")?;
        extract!("data/minect_internal/functions/load.mcfunction")?;
        extract!("data/minect_internal/functions/pulse_redstone.mcfunction")?;
        extract!("data/minect_internal/functions/reload.mcfunction")?;
        extract!("data/minect_internal/functions/tick.mcfunction")?;
        extract!("data/minect_internal/functions/update.mcfunction")?;
        extract!("data/minect/functions/connect/choose_chunk.mcfunction")?;
        extract!("data/minect/functions/disconnect_self.mcfunction")?;
        extract!("data/minect/functions/disconnect.mcfunction")?;
        extract!("data/minect/functions/enable_logging.mcfunction")?;
        extract!("data/minect/functions/reset_logging.mcfunction")?;
        extract!("data/minect/functions/uninstall_completely.mcfunction")?;
        extract!("data/minect/functions/uninstall.mcfunction")?;
        extract!("pack.mcmeta")?;
        Ok(())
    }

    /// Removes the Minect datapack at directory [Self::get_datapack_dir()].
    pub fn remove_datapack(&self) -> Result<(), IoErrorAtPath> {
        remove_dir_all(&self.datapack_dir)
    }

    pub fn inject_commands(
        &mut self,
        commands: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = Command>>,
    ) -> Result<(), InjectCommandsError> {
        if !self.datapack_dir.is_dir() {
            self.create_datapack()?;
        }
        if !self.loaded_listener_initialized {
            self.init_loaded_listener();
        }
        create_dir_all(&self.structures_dir)?;

        let id_path = self.structures_dir.join("id.txt");
        let mut id_file = lock_file(&id_path)?; // Automatically unlocked by dropping id_file at the end of this function.

        let id = read_incremented_id(&mut id_file, &id_path)?;
        let next_id = id.wrapping_add(1);

        let (commands, commands_len) = prepend_loaded_command(id, commands);
        let structure = generate_structure(&self.identifier, next_id, commands, commands_len);

        // To create the structure file as atomically as possible we first write to a temporary file
        // and then rename it, which is an atomic operation on most operating systems. If Minecraft
        // would attempt to load a half written file, it would likely cache the file as invalid
        // (depending on what bytes it sees). Locking the file also causes Minecraft to cache it as
        // invalid.
        let tmp_path = self.get_structure_file("tmp");
        create_structure_file(&tmp_path, structure)?;
        rename(tmp_path, self.get_structure_file(id))?;

        // We do this at the end to not increment the id on a failure, which would break the connection.
        write_id(&mut id_file, id_path, id)?;

        Ok(())
    }

    fn get_structure_file(&self, id: impl Display) -> PathBuf {
        self.structures_dir.join(format!("{}.nbt", id))
    }

    pub fn add_listener(&mut self) -> impl Stream<Item = LogEvent> {
        self.get_log_observer().add_listener()
    }

    pub fn add_named_listener(&mut self, name: impl Into<String>) -> impl Stream<Item = LogEvent> {
        self.get_log_observer().add_named_listener(name)
    }

    fn init_loaded_listener(&mut self) {
        let structures_dir = self.structures_dir.clone();
        let listener = LoadedListener { structures_dir };
        self.get_log_observer().add_loaded_listener(listener);
        self.loaded_listener_initialized = true;
    }

    fn get_log_observer(&mut self) -> &mut LogObserver {
        if self.log_observer.is_none() {
            // Start LogObserver only when needed
            self.log_observer = Some(LogObserver::new(&self.log_file));
        }
        self.log_observer.as_mut().unwrap() // Unwrap is safe because we just assigned the value
    }
}

fn lock_file(path: impl AsRef<Path>) -> Result<File, IoErrorAtPath> {
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(io_error("Failed to open file", path.as_ref()))?;
    file.lock_exclusive()
        .map_err(io_error("Failed to lock file", path.as_ref()))?;
    Ok(file)
}

fn read_incremented_id(file: &mut File, path: impl AsRef<Path>) -> Result<u64, IoErrorAtPath> {
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(io_error("Failed to read file", path.as_ref()))?;
    let id = if content.is_empty() {
        0
    } else {
        content
            .parse::<u64>()
            .map_err(io_invalid_data)
            .map_err(io_error(
                "Failed to parse content as u64 of file",
                path.as_ref(),
            ))?
            .wrapping_add(1)
    };
    Ok(id)
}

fn write_id(file: &mut File, path: impl AsRef<Path>, id: u64) -> Result<(), IoErrorAtPath> {
    file.set_len(0)
        .map_err(io_error("Failed to truncate file", path.as_ref()))?;
    file.seek(SeekFrom::Start(0))
        .map_err(io_error("Failed to seek beginning of file", path.as_ref()))?;
    file.write_all(id.to_string().as_bytes())
        .map_err(io_error("Failed to write to file", path.as_ref()))?;
    Ok(())
}

fn create_structure_file(
    path: impl AsRef<Path>,
    structure: Structure,
) -> Result<(), IoErrorAtPath> {
    let file = create(path)?;
    let mut writer = BufWriter::new(file);
    nbt::to_gzip_writer(&mut writer, &structure, None).unwrap();
    Ok(())
}

#[derive(Debug)]
pub enum InjectCommandsError {
    Io(IoErrorAtPath),
    // TODO: Add error for injecting too many commands instead of ignoring them
}
impl From<IoErrorAtPath> for InjectCommandsError {
    fn from(value: IoErrorAtPath) -> InjectCommandsError {
        InjectCommandsError::Io(value)
    }
}
impl Display for InjectCommandsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InjectCommandsError::Io(error) => error.fmt(f),
        }
    }
}
impl From<InjectCommandsError> for std::io::Error {
    fn from(value: InjectCommandsError) -> std::io::Error {
        match value {
            InjectCommandsError::Io(error) => std::io::Error::from(error),
        }
    }
}

pub struct Command {
    name: Option<String>,
    command: String,
}
impl Command {
    pub fn new(command: impl Into<String>) -> Command {
        Command {
            name: None,
            command: command.into(),
        }
    }

    pub fn named(name: impl Into<String>, command: impl Into<String>) -> Command {
        Command {
            name: Some(name.into()),
            command: command.into(),
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_ref().map(|it| it.as_str())
    }

    pub fn get_command(&self) -> &str {
        &self.command
    }

    fn get_name_as_json(&self) -> Option<String> {
        self.get_name().map(create_json_text_component)
    }
}

struct LoadedListener {
    structures_dir: PathBuf,
}
impl LoadedListener {
    fn on_event(&self, event: LogEvent) {
        if let Some(id) = parse_loaded_output(&event) {
            let structure_file = self.get_structure_file(id);
            if let Err(error) = remove_file(&structure_file) {
                error!("{}", error);
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

const LOADED_LISTENER_NAME: &str = "minect_loaded";
const STRUCTURE_LOADED_OUTPUT_PREFIX: &str = "minect_loaded_";

fn parse_loaded_output(event: &LogEvent) -> Option<u64> {
    if event.executor != LOADED_LISTENER_NAME {
        return None;
    }
    let output = event.output.parse::<SummonNamedEntityOutput>().ok()?;
    let id = &output.name.strip_prefix(STRUCTURE_LOADED_OUTPUT_PREFIX)?;
    id.parse().ok()
}

fn prepend_loaded_command(
    id: u64,
    commands: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = Command>>,
) -> (impl Iterator<Item = Command>, usize) {
    let loaded_cmds = [
        Command::new(enable_logging_command()),
        Command::named(
            LOADED_LISTENER_NAME,
            summon_named_entity_command(&format!("{}{}", STRUCTURE_LOADED_OUTPUT_PREFIX, id)),
        ),
        Command::new(reset_logging_command()),
    ];
    let commands = commands.into_iter();
    let commands_len = loaded_cmds.len() + commands.len();
    let commands = loaded_cmds.into_iter().chain(commands);
    (commands, commands_len)
}
