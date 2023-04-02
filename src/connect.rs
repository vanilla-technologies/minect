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

use crate::{
    command::{summon_named_entity_command, AddTagOutput, SummonNamedEntityOutput},
    io::{create_dir_all, io_error, remove_dir, remove_dir_all, write, IoErrorAtPath},
    log::LogEvent,
    on_drop::OnDrop,
    read_incremented_id, Command, ExecuteCommandsError, ExecuteCommandsErrorInner,
    MinecraftConnection,
};
use indexmap::IndexSet;
use log::error;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Seek, SeekFrom},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio_stream::StreamExt;
use walkdir::WalkDir;

/// The error returned from [MinecraftConnection::connect].
#[derive(Debug)]
pub struct ConnectError {
    inner: ConnectErrorInner,
}
#[derive(Debug)]
enum ConnectErrorInner {
    Io(IoErrorAtPath),
    Cancelled,
}
impl ConnectError {
    fn new(inner: ConnectErrorInner) -> ConnectError {
        ConnectError { inner }
    }

    /// Returns `true` if [connect](MinecraftConnection::connect) failed because the player
    /// cancelled the installation in the interactive installer.
    pub fn is_cancelled(&self) -> bool {
        matches!(self.inner, ConnectErrorInner::Cancelled)
    }
}
impl From<IoErrorAtPath> for ConnectError {
    fn from(value: IoErrorAtPath) -> ConnectError {
        ConnectError::new(ConnectErrorInner::Io(value))
    }
}
impl From<ExecuteCommandsError> for ConnectError {
    fn from(value: ExecuteCommandsError) -> ConnectError {
        match value.inner {
            ExecuteCommandsErrorInner::Io(error) => error.into(),
        }
    }
}
impl Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            ConnectErrorInner::Io(error) => error.fmt(f),
            ConnectErrorInner::Cancelled => write!(f, "Cancelled"),
        }
    }
}
impl std::error::Error for ConnectError {}
impl From<ConnectError> for std::io::Error {
    fn from(value: ConnectError) -> std::io::Error {
        match value.inner {
            ConnectErrorInner::Io(error) => std::io::Error::from(error),
            ConnectErrorInner::Cancelled => {
                std::io::Error::new(std::io::ErrorKind::ConnectionRefused, value)
            }
        }
    }
}

pub(crate) async fn connect(connection: &mut MinecraftConnection) -> Result<(), ConnectError> {
    connection.create_datapack()?;

    let success = AtomicBool::new(false);
    let identifier = connection.identifier.clone();
    let datapack_dir = connection.datapack_dir.clone();
    // Has to be stored to a variable that is not named _ to ensure it is dropped at the end of the function and not right away.
    let _on_drop = OnDrop::new(|| {
        // TODO: use block_on to allow concurrency
        remove_connector(&identifier, &datapack_dir);
        if !success.load(Ordering::Relaxed) {
            remove_disconnector(&identifier, &datapack_dir);
        }
        remove_empty_dirs(&datapack_dir);
    });

    let structure_id = {
        let path = connection.structures_dir.join("id.txt");
        match File::open(&path) {
            Ok(mut file) => read_incremented_id(&mut file, &path)?,
            Err(_) => 0,
        }
    };

    create_connector(&identifier, structure_id, &datapack_dir)?;
    create_disconnector(&identifier, &datapack_dir)?;

    wait_for_connection(connection).await?;
    success.store(true, Ordering::Relaxed);

    Ok(())
}

fn create_connector(
    identifier: &str,
    structure_id: u64,
    datapack_dir: impl AsRef<Path>,
) -> Result<(), IoErrorAtPath> {
    let expand_template = |template: &str| {
        expand_template(template, identifier).replace("-structure_id-", &structure_id.to_string())
    };
    let datapack_dir = datapack_dir.as_ref();

    macro_rules! add_to_function_tag {
        ($relative_path:expr) => {{
            let path = datapack_dir.join($relative_path);
            let template = expand_template(include_datapack_template!($relative_path));
            add_to_function_tag(path, &template)
        }};
    }
    add_to_function_tag!("data/minect_internal/tags/functions/connect/choose_chunk.json")?;
    add_to_function_tag!("data/minect_internal/tags/functions/connect/prompt.json")?;

    macro_rules! expand {
        ($relative_path:expr) => {{
            let path = datapack_dir.join(expand_template($relative_path));
            let contents = expand_template(include_datapack_template!($relative_path));
            write(path, &contents)
        }};
    }
    expand!("data/minect_internal/functions/connection/-connection_id-/connect/cancel_cleanup.mcfunction")?;
    expand!("data/minect_internal/functions/connection/-connection_id-/connect/cancel.mcfunction")?;
    expand!(
        "data/minect_internal/functions/connection/-connection_id-/connect/choose_chunk_unchecked.mcfunction"
    )?;
    expand!(
        "data/minect_internal/functions/connection/-connection_id-/connect/choose_chunk.mcfunction"
    )?;
    expand!("data/minect_internal/functions/connection/-connection_id-/connect/confirm_chunk.mcfunction")?;
    expand!("data/minect_internal/functions/connection/-connection_id-/connect/prompt_unchecked.mcfunction")?;
    expand!("data/minect_internal/functions/connection/-connection_id-/connect/prompt.mcfunction")?;

    Ok(())
}

fn remove_connector(identifier: &str, datapack_dir: impl AsRef<Path>) {
    let expand_template = |template: &str| expand_template(template, identifier);
    let datapack_dir = datapack_dir.as_ref();

    macro_rules! remove_from_function_tag {
        ($relative_path:expr) => {{
            let path = datapack_dir.join($relative_path);
            let template = expand_template(include_datapack_template!($relative_path));
            log_cleanup_error(remove_from_function_tag(path, &template))
        }};
    }
    remove_from_function_tag!("data/minect_internal/tags/functions/connect/choose_chunk.json");
    remove_from_function_tag!("data/minect_internal/tags/functions/connect/prompt.json");

    let remove = |template_path| {
        let path = datapack_dir.join(expand_template(template_path));
        log_cleanup_error(remove_dir_all(path));
    };
    remove("data/minect_internal/functions/connection/-connection_id-/connect");
}

fn create_disconnector(
    identifier: &str,
    datapack_dir: impl AsRef<Path>,
) -> Result<(), IoErrorAtPath> {
    let expand_template = |template: &str| expand_template(template, identifier);
    let datapack_dir = datapack_dir.as_ref();

    macro_rules! add_to_function_tag {
        ($relative_path:expr) => {{
            let path = datapack_dir.join($relative_path);
            let template = expand_template(include_datapack_template!($relative_path));
            add_to_function_tag(path, &template)
        }};
    }
    add_to_function_tag!("data/minect_internal/tags/functions/disconnect/prompt.json")?;

    macro_rules! expand {
        ($relative_path:expr) => {{
            let path = datapack_dir.join(expand_template($relative_path));
            let contents = expand_template(include_datapack_template!($relative_path));
            write(path, &contents)
        }};
    }
    expand!(
        "data/minect_internal/functions/connection/-connection_id-/disconnect/prompt.mcfunction"
    )?;

    Ok(())
}

fn remove_disconnector(identifier: &str, datapack_dir: impl AsRef<Path>) {
    let expand_template = |template: &str| expand_template(template, identifier);
    let datapack_dir = datapack_dir.as_ref();

    macro_rules! remove_from_function_tag {
        ($relative_path:expr) => {{
            let path = datapack_dir.join($relative_path);
            let template = expand_template(include_datapack_template!($relative_path));
            log_cleanup_error(remove_from_function_tag(path, &template))
        }};
    }
    remove_from_function_tag!("data/minect_internal/tags/functions/disconnect/prompt.json");

    let remove = |template_path| {
        let path = datapack_dir.join(expand_template(template_path));
        log_cleanup_error(remove_dir_all(path));
    };
    remove("data/minect_internal/functions/connection/-connection_id-/disconnect");
}

fn remove_empty_dirs(datapack_dir: impl AsRef<Path>) {
    for entry in WalkDir::new(&datapack_dir).contents_first(true) {
        if let Ok(entry) = entry {
            if entry.file_type().is_dir() {
                let _ = remove_dir(entry.path());
            }
        }
    }
}

// If we fail to clean something up we still want to try to clean up the rest
fn log_cleanup_error(result: Result<(), impl Display>) {
    if let Err(e) = result {
        error!("Failed to clean up after connect: {}", e)
    }
}

fn expand_template(template: &str, identifier: &str) -> String {
    template.replace("-connection_id-", identifier)
}

async fn wait_for_connection(connection: &mut MinecraftConnection) -> Result<(), ConnectError> {
    const CONNECT_OUTPUT_PREFIX: &str = "minect_connect_";
    const LISTENER_NAME: &str = "minect_connect";

    let events = connection.add_named_listener(LISTENER_NAME);

    connection.execute_commands([Command::named(
        LISTENER_NAME,
        summon_named_entity_command(&format!("{}success", CONNECT_OUTPUT_PREFIX)),
    )])?;

    enum Output {
        Success,
        Cancelled,
    }
    impl TryFrom<LogEvent> for Output {
        type Error = ();
        fn try_from(event: LogEvent) -> Result<Self, Self::Error> {
            let output = if let Ok(output) = event.output.parse::<SummonNamedEntityOutput>() {
                output.name
            } else if let Ok(output) = event.output.parse::<AddTagOutput>() {
                output.tag
            } else {
                return Err(());
            };

            match output.strip_prefix(CONNECT_OUTPUT_PREFIX) {
                Some("success") => Ok(Output::Success),
                Some("cancelled") => Ok(Output::Cancelled),
                _ => Err(()),
            }
        }
    }
    let output = events
        .filter_map(|event| event.try_into().ok())
        .next()
        .await;
    match output.expect("LogObserver panicked") {
        Output::Success => Ok(()),
        Output::Cancelled => Err(ConnectError::new(ConnectErrorInner::Cancelled)),
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct FunctionTag {
    #[serde(default)]
    values: IndexSet<String>,
}
impl FunctionTag {
    fn new() -> FunctionTag {
        FunctionTag {
            values: IndexSet::new(),
        }
    }
}

fn add_to_function_tag(path: impl AsRef<Path>, template: &str) -> Result<(), IoErrorAtPath> {
    let tag_template: FunctionTag = serde_json::from_str(template).unwrap(); // Our templates are valid, so this can't fail

    modify_function_tag(path, |tag| {
        tag.values.extend(tag_template.values);
    })
}

fn remove_from_function_tag(path: impl AsRef<Path>, template: &str) -> Result<(), IoErrorAtPath> {
    let tag_template: FunctionTag = serde_json::from_str(template).unwrap(); // Our templates are valid, so this can't fail

    modify_function_tag(path, |tag| {
        tag.values
            .retain(|value| !tag_template.values.contains(value))
    })
}

fn modify_function_tag(
    path: impl AsRef<Path>,
    modify: impl FnOnce(&mut FunctionTag),
) -> Result<(), IoErrorAtPath> {
    let (mut tag, mut file) = read_function_tag(&path)?;
    let old_len = tag.values.len();
    modify(&mut tag);
    let modified = old_len != tag.values.len();
    if modified {
        write_function_tag(&tag, &mut file, path)?;
    }
    Ok(())
}

fn read_function_tag(path: &impl AsRef<Path>) -> Result<(FunctionTag, File), IoErrorAtPath> {
    if let Some(parent) = path.as_ref().parent() {
        create_dir_all(parent)?;
    }
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
        .map_err(io_error("Failed to open file", path.as_ref()))?;
    let reader = BufReader::new(&file);
    let tag = match serde_json::from_reader(reader) {
        Ok(tag) => tag,
        Err(e) if e.is_eof() && e.line() == 1 && e.column() == 0 => FunctionTag::new(),
        Err(e) => return Err(IoErrorAtPath::new("Failed to parse file", path.as_ref(), e)),
    };
    Ok((tag, file))
}

fn write_function_tag(
    tag: &FunctionTag,
    file: &mut File,
    path: impl AsRef<Path>,
) -> Result<(), IoErrorAtPath> {
    file.set_len(0)
        .map_err(io_error("Failed to truncate file", path.as_ref()))?;
    file.seek(SeekFrom::Start(0))
        .map_err(io_error("Failed to seek beginning of file", path.as_ref()))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, tag)
        .map_err(io_error("Failed to write to file", path.as_ref()))
}
