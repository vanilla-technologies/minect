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

use minect::{
    command::{summon_named_entity_command, SummonNamedEntityOutput},
    Command, MinecraftConnection,
};
use simple_logger::SimpleLogger;
use std::{
    io,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    fs::{create_dir_all, write},
    sync::OnceCell,
    time::{error::Elapsed, timeout},
};
use tokio_stream::StreamExt;

const TEST_WORLD_DIR: &str = env!("TEST_WORLD_DIR");
const TEST_LOG_FILE: &str = env!("TEST_LOG_FILE");
const INITIAL_CONNECT_TIMEOUT: Duration = Duration::from_secs(60);
pub const TEST_TIMEOUT: Duration = Duration::from_secs(10);

pub fn new_connection() -> MinecraftConnection {
    MinecraftConnection::builder("test", TEST_WORLD_DIR)
        .log_file(TEST_LOG_FILE)
        .build()
}

async fn before_all_tests() {
    SimpleLogger::new().init().unwrap();

    // If this is the first connection to Minecraft we need to reload to activate the minect datapack.
    let mut connection = new_connection();
    connection
        .execute_commands([Command::new("reload")])
        .unwrap();
    wait_for_connection(&mut connection).await.unwrap();
}

async fn wait_for_connection(
    connection: &mut MinecraftConnection,
) -> Result<Option<SummonNamedEntityOutput>, Elapsed> {
    const INITIAL_CONNECT_ENTITY_NAME: &str = "test_connected";
    let commands = [Command::new(summon_named_entity_command(
        INITIAL_CONNECT_ENTITY_NAME,
    ))];
    let events = connection.add_listener();
    let mut events = events
        .filter_map(|event| event.output.parse::<SummonNamedEntityOutput>().ok())
        .filter(|output| output.name == INITIAL_CONNECT_ENTITY_NAME);
    connection.execute_commands(commands).unwrap();

    timeout(INITIAL_CONNECT_TIMEOUT, events.next()).await
}

pub async fn before_each_test() {
    static BEFORE_ALL_TESTS: OnceCell<()> = OnceCell::const_new();
    BEFORE_ALL_TESTS.get_or_init(before_all_tests).await;
}

fn test_datapack_dir() -> PathBuf {
    Path::new(TEST_WORLD_DIR)
        .join("datapacks")
        .join("minect-test")
}

pub async fn create_test_pack_mcmeta() -> io::Result<()> {
    let path = test_datapack_dir().join("pack.mcmeta");
    let contents = r#"{"pack":{"pack_format":7,"description":"Minect test"}}"#;
    create_parent_dir_all(&path).await?;
    write(path, contents).await
}

pub async fn create_test_function(name: &str, commands: &[String]) -> io::Result<()> {
    let (namespace, path) = name.split_once(':').unwrap();
    let path = test_datapack_dir()
        .join("data")
        .join(namespace)
        .join("functions")
        .join(path)
        .with_extension("mcfunction");
    let contents = commands.join("\n");
    create_parent_dir_all(&path).await?;
    write(path, contents).await
}

async fn create_parent_dir_all(path: &PathBuf) -> Result<(), io::Error> {
    if let Some(parent) = path.parent() {
        create_dir_all(&parent).await?;
    }
    Ok(())
}
