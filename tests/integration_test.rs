use futures::executor::block_on;
use log::LevelFilter;
use minect::{
    log::{
        add_tag_command, enable_logging_command, logged_command, named_logged_command,
        query_scoreboard_command, reset_logging_command, summon_named_entity_command, AddTagOutput,
        QueryScoreboardOutput, SummonNamedEntityOutput,
    },
    MinecraftConnection,
};
use serial_test::serial;
use simplelog::{Config, SimpleLogger};
use std::{io, sync::Once, time::Duration};
use tokio::time::timeout;
use tokio_stream::StreamExt;

const TEST_WORLD_DIR: &str = env!("TEST_WORLD_DIR");
const TEST_LOG_FILE: &str = env!("TEST_LOG_FILE");

fn new_connection() -> MinecraftConnection {
    MinecraftConnection::builder("test", TEST_WORLD_DIR)
        .log_file(TEST_LOG_FILE)
        .build()
}

static BEFORE_ALL_TESTS: Once = Once::new();

fn before_all_tests() {
    SimpleLogger::init(LevelFilter::Trace, Config::default()).unwrap();

    let mut connection = new_connection();
    eprintln!("If you are connecting for the first time please execute /reload in Minecraft.");
    block_on(connection.connect()).unwrap();
}

fn before_test() {
    BEFORE_ALL_TESTS.call_once(|| before_all_tests());
}

#[tokio::test]
#[serial]
async fn test_add_tag_command() -> io::Result<()> {
    before_test();
    // given:
    let mut connection = new_connection();
    let listener_name = "test";
    let tag = "success";
    let commands = [
        "say running test_add_tag_command",
        &logged_command(enable_logging_command()),
        &named_logged_command(listener_name, add_tag_command("@s", tag)),
        &logged_command(reset_logging_command()),
    ];
    let mut events = connection.add_named_listener(listener_name);

    // when:
    connection.inject_commands(&commands)?;

    // then:
    let event = timeout(Duration::from_secs(5), events.next())
        .await?
        .unwrap();
    let output = event.output.parse::<AddTagOutput>().unwrap();
    assert_eq!(output.tag, tag);
    assert_eq!(output.entity, listener_name);
    assert_eq!(output.to_string(), "Added tag 'success' to test");

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_summon_named_entity_command() -> io::Result<()> {
    before_test();
    // given:
    let mut connection = new_connection();
    let name = "success";
    let commands = [
        "say running test_summon_named_entity_command",
        &enable_logging_command(),
        &summon_named_entity_command(name),
        &reset_logging_command(),
    ];
    let events = connection.add_listener();

    // when:
    connection.inject_commands(&commands)?;

    // then:
    let output = timeout(
        Duration::from_secs(5),
        events
            .filter_map(|event| event.output.parse::<SummonNamedEntityOutput>().ok())
            .filter(|output| output.name == name)
            .next(),
    )
    .await?
    .unwrap();
    assert_eq!(output.to_string(), "Summoned new success");

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_query_scoreboard_command() -> io::Result<()> {
    before_test();
    // given:
    let mut connection = new_connection();
    let scoreboard = "minect_test_global";
    let entity = "@e[type=sheep,tag=minect_test_sheep]";
    let commands = [
        "say running test_query_scoreboard_command",
        &format!("scoreboard objectives add {} dummy", scoreboard),
        "summon sheep ~ ~ ~ {Tags:[minect_test_sheep],NoAI:true}",
        &format!("scoreboard players set {} {} 42", entity, scoreboard),
        &enable_logging_command(),
        &query_scoreboard_command(entity, scoreboard),
        &reset_logging_command(),
        &format!("kill {}", entity),
        &format!("scoreboard objectives remove {}", scoreboard),
    ];
    let events = connection.add_listener();

    // when:
    connection.inject_commands(&commands)?;

    // then:
    let output = timeout(
        Duration::from_secs(5),
        events
            .filter_map(|event| event.output.parse::<QueryScoreboardOutput>().ok())
            .next(),
    )
    .await?
    .unwrap();
    assert_eq!(output.scoreboard, scoreboard);
    assert_eq!(output.score, 42);

    Ok(())
}
