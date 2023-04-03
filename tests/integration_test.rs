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

mod utils;

use crate::utils::{
    before_each_test, create_test_function, create_test_pack_mcmeta, new_connection, TEST_TIMEOUT,
};
use minect::{
    command::{
        add_tag_command, enable_logging_command, logged_block_commands, logged_cart_command,
        named_logged_block_commands, named_logged_cart_command, query_scoreboard_command,
        reset_logging_command, summon_named_entity_command, AddTagOutput, QueryScoreboardOutput,
        SummonNamedEntityOutput,
    },
    Command,
};
use serial_test::serial;
use std::io;
use tokio::time::timeout;
use tokio_stream::StreamExt;

#[tokio::test]
#[serial]
async fn test_summon_named_entity_command() -> io::Result<()> {
    before_each_test().await;
    // given:
    let mut connection = new_connection();
    let listener_name = "test";
    let name = "success";
    let commands = [
        Command::new("say running test_summon_named_entity_command"),
        Command::named(listener_name, summon_named_entity_command(name)),
    ];
    let mut events = connection.add_named_listener(listener_name);

    // when:
    connection.execute_commands(commands)?;

    // then:
    let event = timeout(TEST_TIMEOUT, events.next()).await?.unwrap();
    let output = event.output.parse::<SummonNamedEntityOutput>().unwrap();
    assert_eq!(output.name, name);

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_query_scoreboard_command() -> io::Result<()> {
    before_each_test().await;
    // given:
    let mut connection = new_connection();
    let listener_name = "test";
    let scoreboard = "minect_test";
    let entity = "@e[type=sheep,tag=minect_test_sheep]";
    let commands = [
        Command::new("say running test_query_scoreboard_command"),
        Command::new(format!("scoreboard objectives add {} dummy", scoreboard)),
        Command::new("summon sheep ~ ~ ~ {Tags:[minect_test_sheep],NoAI:true}"),
        Command::new(format!(
            "scoreboard players set {} {} 42",
            entity, scoreboard
        )),
        Command::named(listener_name, query_scoreboard_command(entity, scoreboard)),
        Command::new(format!("kill {}", entity)),
        Command::new(format!("scoreboard objectives remove {}", scoreboard)),
    ];
    let mut events = connection.add_named_listener(listener_name);

    // when:
    connection.execute_commands(commands)?;

    // then:
    let event = timeout(TEST_TIMEOUT, events.next()).await?.unwrap();
    let output = event.output.parse::<QueryScoreboardOutput>().unwrap();
    assert_eq!(output.scoreboard, scoreboard);
    assert_eq!(output.score, 42);

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_add_tag_command() -> io::Result<()> {
    before_each_test().await;
    // given:
    let mut connection = new_connection();
    let listener_name = "test";
    let tag = "success";
    let commands = [
        Command::new("say running test_add_tag_command"),
        Command::new(logged_cart_command(enable_logging_command())),
        Command::new(named_logged_cart_command(
            listener_name,
            add_tag_command("@s", tag),
        )),
        Command::new(logged_cart_command(reset_logging_command())),
    ];
    let mut events = connection.add_named_listener(listener_name);

    // when:
    connection.execute_commands(commands)?;

    // then:
    let event = timeout(TEST_TIMEOUT, events.next()).await?.unwrap();
    let output = event.output.parse::<AddTagOutput>().unwrap();
    assert_eq!(output.tag, tag);
    assert_eq!(output.entity, listener_name);
    assert_eq!(output.to_string(), "Added tag 'success' to test");

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_logged_block_commands_are_executed_within_the_same_tick() -> io::Result<()> {
    before_each_test().await;
    // given:
    let mut connection = new_connection();
    let listener_name = "test";

    create_test_pack_mcmeta().await?;
    let fn_name = "minect_test:test";

    let mut fn_commands = Vec::from_iter([
        "scoreboard objectives add minect_test dummy".to_string(),
        "execute store result score gametime1 minect_test run time query gametime".to_string(),
    ]);

    let command = "execute store result score gametime2 minect_test run time query gametime";
    fn_commands.extend(logged_block_commands(command));

    let command = query_scoreboard_command("gametime1", "minect_test");
    fn_commands.extend(named_logged_block_commands(listener_name, &command));

    let command = query_scoreboard_command("gametime2", "minect_test");
    fn_commands.extend(named_logged_block_commands(listener_name, &command));

    create_test_function(fn_name, &fn_commands).await?;
    connection.execute_commands([Command::new("reload")])?;

    let mut events = connection.add_named_listener(listener_name);
    let commands = [
        Command::new("say running test_logged_block_commands_are_executed_within_the_same_tick"),
        Command::new(format!("function {}", fn_name)),
    ];

    // when:
    connection.execute_commands(commands)?;

    // then:
    let event1 = timeout(TEST_TIMEOUT, events.next()).await?.unwrap();
    let event2 = timeout(TEST_TIMEOUT, events.next()).await?.unwrap();
    let output1 = event1.output.parse::<QueryScoreboardOutput>().unwrap();
    let output2 = event2.output.parse::<QueryScoreboardOutput>().unwrap();
    assert_eq!(output1.entity, "gametime1");
    assert_eq!(output2.entity, "gametime2");
    assert_eq!(output1.score, output2.score);

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_scheduled_logged_block_commands_are_executed_within_the_same_tick() -> io::Result<()>
{
    before_each_test().await;
    // given:
    let mut connection = new_connection();
    let listener_name = "test";

    create_test_pack_mcmeta().await?;
    let fn_name = "minect_test:test";

    let mut fn_commands = Vec::from_iter([
        "scoreboard objectives add minect_test dummy".to_string(),
        "execute store result score gametime1 minect_test run time query gametime".to_string(),
    ]);

    let command = "execute store result score gametime2 minect_test run time query gametime";
    fn_commands.extend(logged_block_commands(command));

    let command = query_scoreboard_command("gametime1", "minect_test");
    fn_commands.extend(named_logged_block_commands(listener_name, &command));

    let command = query_scoreboard_command("gametime2", "minect_test");
    fn_commands.extend(named_logged_block_commands(listener_name, &command));

    create_test_function(fn_name, &fn_commands).await?;
    connection.execute_commands([Command::new("reload")])?;

    let mut events = connection.add_named_listener(listener_name);
    let commands = [
        Command::new(
            "say running test_scheduled_logged_block_commands_are_executed_within_the_same_tick",
        ),
        Command::new(format!("schedule function {} 1", fn_name)),
    ];

    // when:
    connection.execute_commands(commands)?;

    // then:
    let event1 = timeout(TEST_TIMEOUT, events.next()).await?.unwrap();
    let event2 = timeout(TEST_TIMEOUT, events.next()).await?.unwrap();
    let output1 = event1.output.parse::<QueryScoreboardOutput>().unwrap();
    let output2 = event2.output.parse::<QueryScoreboardOutput>().unwrap();
    assert_eq!(output1.entity, "gametime1");
    assert_eq!(output2.entity, "gametime2");
    assert_eq!(output1.score, output2.score);

    Ok(())
}
