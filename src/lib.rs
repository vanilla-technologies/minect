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

//! Minect is a library that allows a Rust program to connect to a running Minecraft instance
//! without requiring any Minecraft mods.
//!
//! Using Minect a Rust program can execute commands in Minecraft and listen for command output. This
//! way a Rust program can control or be controlled by Minecraft.
//!
//! The connection requires a building in Minecraft which continuously loads structure files that
//! contain the commands generated by the Rust program. Listening for their output works by polling
//! Minecraft's log file.
//!
//! ## Example
//!
//! ```no_run
//! # use minect::*;
//! # use minect::command::*;
//! # use tokio_stream::StreamExt;
//! # let _ = async {
//! let identifier = "MyProgram";
//! let world_dir = "C:/Users/Herobrine/AppData/Roaming/.minecraft/saves/New World";
//! let mut connection = MinecraftConnection::builder(identifier, world_dir).build();
//!
//! println!("If you are connecting for the first time please execute /reload in Minecraft.");
//! connection.connect().await?;
//!
//! let events = connection.add_listener();
//!
//! connection.execute_commands([
//!   Command::new("scoreboard objectives add example dummy"),
//!   Command::new("scoreboard players set Herobrine example 42"),
//!   Command::new(query_scoreboard_command("Herobrine", "example")),
//! ])?;
//!
//! let output = events
//!   .filter_map(|event| event.output.parse::<QueryScoreboardOutput>().ok())
//!   .next()
//!   .await
//!   .expect("Minecraft connection was closed unexpectedly");
//!
//! println!("{}'s score is {}", output.entity, output.score);
//! # Ok::<(), std::io::Error>(())
//! # };
//! ```

#[macro_use]
mod macros;

pub mod command;
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
    command::{
        enable_logging_command, reset_logging_command, summon_named_entity_command,
        SummonNamedEntityOutput,
    },
    connect::connect,
    io::{
        create, create_dir_all, io_error, remove_dir_all, remove_file, rename, write, IoErrorAtPath,
    },
    log::{LogEvent, LogObserver},
    placement::generate_structure,
    structure::nbt::Structure,
    utils::io_invalid_data,
};
use ::log::error;
use fs3::FileExt;
use indexmap::IndexSet;
use json::create_json_text_component;
use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use tokio_stream::Stream;

/// A builder to create a [MinecraftConnection] is obtained via [MinecraftConnection::builder].
///
/// The builder pattern is used to add new parameters without breaking backwards compatibility.
pub struct MinecraftConnectionBuilder {
    identifier: String,
    world_dir: PathBuf,
    log_file: Option<PathBuf>,
    enable_logging_automatically: bool,
}

impl MinecraftConnectionBuilder {
    fn new(
        identifier: impl Into<String>,
        world_dir: impl Into<PathBuf>,
    ) -> MinecraftConnectionBuilder {
        let identifier = identifier.into();
        validate_identifier(&identifier);
        MinecraftConnectionBuilder {
            identifier,
            world_dir: world_dir.into(),
            log_file: None,
            enable_logging_automatically: true,
        }
    }

    /// The path to Minecraft's log file.
    ///
    /// For single player this is typically at these locations:
    /// * Windows: `C:\Users\Herobrine\AppData\Roaming\.minecraft\logs\latest.log`
    /// * GNU/Linux: `~/.minecraft/logs/latest.log`
    /// * Mac: `~/Library/Application Support/minecraft/logs/latest.log`
    ///
    /// For servers it is at `logs/latest.log` in the server directory.
    ///
    /// Defaults to `../../logs/latest.log` relative to `world_dir`, which is the correct value for
    /// single player, but usually not for servers.
    pub fn log_file(mut self, log_file: impl Into<PathBuf>) -> MinecraftConnectionBuilder {
        self.log_file = Some(log_file.into());
        self
    }

    /// Whether logging is automatically enabled for all commands passed to
    /// [MinecraftConnection::execute_commands]. This works by prepending an
    /// [enable_logging_command] and appending a [reset_logging_command] to the list of commands.
    ///
    /// This setting has no effect for [logged_command](command::logged_command)s. Logging still has
    /// to be enabled for these manually.
    ///
    /// Default: `true`.
    pub fn enable_logging_automatically(
        mut self,
        enable_logging_automatically: impl Into<bool>,
    ) -> MinecraftConnectionBuilder {
        self.enable_logging_automatically = enable_logging_automatically.into();
        self
    }

    /// Creates a [MinecraftConnection] with the configured parameters.
    ///
    /// # Panics
    ///
    /// Panics if no [log_file](Self::log_file()) was specified and the
    /// [world_dir](MinecraftConnection::builder) has less than 2 path compontents. In this case the
    /// default value of `../../logs/latest.log` can not be resolved.
    pub fn build(self) -> MinecraftConnection {
        let world_dir = self.world_dir;
        let log_file = self
            .log_file
            .unwrap_or_else(|| log_file_from_world_dir(&world_dir));
        MinecraftConnection::new(
            self.identifier,
            world_dir,
            log_file,
            self.enable_logging_automatically,
        )
    }
}

fn validate_identifier(identifier: &str) {
    let invalid_chars = identifier
        .chars()
        .filter(|c| !is_allowed_in_identifier(*c))
        .collect::<IndexSet<_>>();
    if !invalid_chars.is_empty() {
        panic!(
            "Invalid characters in MinecraftConnection.identifier: '{}'",
            invalid_chars
                .iter()
                .fold(String::new(), |joined, c| joined + &c.to_string())
        );
    }
}
fn is_allowed_in_identifier(c: char) -> bool {
    return c >= '0' && c <= '9'
        || c >= 'A' && c <= 'Z'
        || c >= 'a' && c <= 'z'
        || c == '+'
        || c == '-'
        || c == '.'
        || c == '_';
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

/// A connection to Minecraft that can [execute commands](MinecraftConnection::execute_commands) in
/// Minecraft and [listen for command output](MinecraftConnection::add_listener).
///
/// If you need to listen for command output, but don't need to execute commands, consider using
/// [LogObserver] directly.
///
/// The connection requires a building in Minecraft which continuously loads structure files that
/// contain the commands passed to [execute_commands](MinecraftConnection::execute_commands). To
/// create such a connection building you can call [connect](MinecraftConnection::connect).
///
/// Minect supports operating multiple connection buildings in parallel, each with a unique
/// identifier. A single connection building can be shared between any number of Rust programs, but
/// it has a limited update frequency. For optimal performance every Rust program can use a
/// different connection identifier.
///
/// The update frequency can be configured globally for all connections in a Minecraft world by
/// changing the score of `update_delay` for the objective `minect_config`.
pub struct MinecraftConnection {
    identifier: String,
    structures_dir: PathBuf,
    datapack_dir: PathBuf,
    log_file: PathBuf,
    log_observer: Option<LogObserver>,
    loaded_listener_initialized: bool,
    enable_logging_automatically: bool,
    _private: (),
}

const NAMESPACE: &str = "minect";

impl MinecraftConnection {
    /// Creates a [MinecraftConnectionBuilder].
    ///
    /// `identifier` is a string that uniquely identifies a connection building in Minecraft.
    /// Because it is used with Minecraft's `tag` command, it may only contain the following
    /// Characters: `0-9`, `A-Z`, `a-z`, `+`, `-`, `.` & `_`.
    ///
    /// `world_dir` is the directory containing the Minecraft world to connect to.
    /// For single player this is typically a directory within the saves directory:
    /// * Windows: `C:\Users\Herobrine\AppData\Roaming\.minecraft\saves\`
    /// * GNU/Linux: `~/.minecraft/saves/`
    /// * Mac: `~/Library/Application Support/minecraft/saves/`
    ///
    /// For servers it is specified in `server.properties`.
    ///
    /// # Panics
    ///
    /// Panics if `identifier` contains an invalid character.
    pub fn builder(
        identifier: impl Into<String>,
        world_dir: impl Into<PathBuf>,
    ) -> MinecraftConnectionBuilder {
        MinecraftConnectionBuilder::new(identifier, world_dir)
    }

    fn new(
        identifier: String,
        world_dir: PathBuf,
        log_file: PathBuf,
        enable_logging_automatically: bool,
    ) -> MinecraftConnection {
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
            enable_logging_automatically,
            _private: (),
        }
    }

    /// The connection identifier uniquely identifies a connection building in Minecraft.
    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }

    /// The root directory of the datapack used to operate the connection in Minecraft.
    pub fn get_datapack_dir(&self) -> &Path {
        &self.datapack_dir
    }

    /// This function can be used to set up the connection building in Minecraft, which is required
    /// for [execute_commands](Self::execute_commands).
    ///
    /// If the connection can be established, this function simply returns. Note that this still
    /// requires a running Minecraft instance. Otherwise this function blocks until a connection
    /// building is created.
    /// This function also creates an interactive installer that a player can start by executing
    /// `/reload` in Minecraft.
    ///
    /// Because this function blocks indefinately if the connection can't be established, it should
    /// be called with [tokio::time::timeout] or some other means of cancellation, such as
    /// [futures::future::select].
    ///
    /// # Errors
    ///
    /// This function will return an error if the player cancels the installation in the interactive
    /// installer (can be checked with [ConnectError::is_cancelled]) or if an
    /// [io::Error](std::io::Error) occurs.
    pub async fn connect(&mut self) -> Result<(), ConnectError> {
        connect(self).await
    }

    /// Creates the [Minect datapack](Self::get_datapack_dir()).
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
        extract!("data/minect_internal/functions/enable_logging_initially.mcfunction")?;
        extract!("data/minect_internal/functions/load.mcfunction")?;
        extract!("data/minect_internal/functions/pulse_redstone.mcfunction")?;
        extract!("data/minect_internal/functions/reload.mcfunction")?;
        extract!("data/minect_internal/functions/reset_logging_finally.mcfunction")?;
        extract!("data/minect_internal/functions/tick.mcfunction")?;
        extract!("data/minect_internal/functions/update.mcfunction")?;
        extract!("data/minect_internal/functions/v1_uninstall.mcfunction")?;
        extract!("data/minect_internal/functions/v2_install.mcfunction")?;
        extract!("data/minect_internal/functions/v2_uninstall.mcfunction")?;
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

    /// Removes the [Minect datapack](Self::get_datapack_dir()).
    pub fn remove_datapack(&self) -> Result<(), IoErrorAtPath> {
        remove_dir_all(&self.datapack_dir)
    }

    /// Executes the given `commands` in Minecraft.
    ///
    /// # Errors
    ///
    /// This function will return an error if an [io::Error](std::io::Error) occurs.
    pub fn execute_commands(
        &mut self,
        commands: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = Command>>,
    ) -> Result<(), ExecuteCommandsError> {
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

        let (commands, commands_len) =
            add_implicit_commands(id, commands, self.enable_logging_automatically);
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

    /// Returns a [Stream] of all [LogEvent]s. To remove the listener simply drop the stream.
    ///
    /// Internally the stream is backed by an unbound channel. This means it should be polled
    /// regularly to avoid memory leaks.
    pub fn add_listener(&mut self) -> impl Stream<Item = LogEvent> {
        self.get_log_observer().add_listener()
    }

    /// Returns a [Stream] of [LogEvent]s with [executor](LogEvent::executor) equal to the given
    /// `name`. To remove the listener simply drop the stream.
    ///
    /// This can be more memory efficient than [add_listener](Self::add_listener), because only a
    /// small subset of [LogEvent]s has to be buffered if not that many commands are executed with
    /// the given `name`.
    ///
    /// Internally the stream is backed by an unbound channel. This means it should be polled
    /// regularly to avoid memory leaks.
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

/// The error returned from [MinecraftConnection::execute_commands].
#[derive(Debug)]
pub struct ExecuteCommandsError {
    inner: ExecuteCommandsErrorInner,
}
#[derive(Debug)]
enum ExecuteCommandsErrorInner {
    Io(IoErrorAtPath),
    // TODO: Add error for executing too many commands instead of ignoring them
}
impl ExecuteCommandsError {
    fn new(inner: ExecuteCommandsErrorInner) -> ExecuteCommandsError {
        ExecuteCommandsError { inner }
    }
}
impl From<IoErrorAtPath> for ExecuteCommandsError {
    fn from(value: IoErrorAtPath) -> ExecuteCommandsError {
        ExecuteCommandsError::new(ExecuteCommandsErrorInner::Io(value))
    }
}
impl Display for ExecuteCommandsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            ExecuteCommandsErrorInner::Io(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for ExecuteCommandsError {}
impl From<ExecuteCommandsError> for std::io::Error {
    fn from(value: ExecuteCommandsError) -> std::io::Error {
        match value.inner {
            ExecuteCommandsErrorInner::Io(error) => std::io::Error::from(error),
        }
    }
}

/// A [Command] can be passed to [MinecraftConnection::execute_commands] and contains a Minecraft
/// command to execute and optionally a custom name.
///
/// The custom name can be useful in conjunction with [MinecraftConnection::add_named_listener] to
/// easily and performantly filter for the correct [LogEvent].
pub struct Command {
    name: Option<String>,
    command: String,
}
impl Command {
    /// Creates a [Command] without custom name. These commands are typically executed under the
    /// name `@`.
    pub fn new(command: impl Into<String>) -> Command {
        Command {
            name: None,
            command: command.into(),
        }
    }

    /// Creates a [Command] with the given custom `name`.
    pub fn named(name: impl Into<String>, command: impl Into<String>) -> Command {
        Command {
            name: Some(name.into()),
            command: command.into(),
        }
    }

    /// The optional custom name.
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_ref().map(|it| it.as_str())
    }

    /// The Minecraft command.
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

fn add_implicit_commands(
    id: u64,
    commands: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = Command>>,
    enable_logging_automatically: bool,
) -> (impl Iterator<Item = Command>, usize) {
    let mut first_cmds = Vec::from_iter([
        Command::new(enable_logging_command()),
        Command::named(
            LOADED_LISTENER_NAME,
            summon_named_entity_command(&format!("{}{}", STRUCTURE_LOADED_OUTPUT_PREFIX, id)),
        ),
    ]);
    let mut last_cmds = Vec::new();
    if !enable_logging_automatically {
        first_cmds.push(Command::new(reset_logging_command()));
    } else {
        last_cmds.push(Command::new(reset_logging_command()));
    }

    let commands = commands.into_iter();
    let commands_len = first_cmds.len() + commands.len();
    let commands = first_cmds.into_iter().chain(commands).chain(last_cmds);
    (commands, commands_len)
}
