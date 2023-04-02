use log::LevelFilter;
use minect::{
    command::{
        add_tag_command, enable_logging_command, logged_command, named_logged_command,
        query_scoreboard_command, reset_logging_command, summon_named_entity_command, AddTagOutput,
        QueryScoreboardOutput, SummonNamedEntityOutput,
    },
    Command, MinecraftConnection,
};
use serial_test::serial;
use simplelog::{Config, SimpleLogger};
use std::{io, time::Duration};
use tokio::{sync::OnceCell, time::timeout};
use tokio_stream::StreamExt;

const TEST_WORLD_DIR: &str = env!("TEST_WORLD_DIR");
const TEST_LOG_FILE: &str = env!("TEST_LOG_FILE");

fn new_connection() -> MinecraftConnection {
    MinecraftConnection::builder("test", TEST_WORLD_DIR)
        .log_file(TEST_LOG_FILE)
        .build()
}

async fn before_all_tests() {
    SimpleLogger::init(LevelFilter::Trace, Config::default()).unwrap();

    eprintln!("If you are connecting for the first time please execute /reload in Minecraft.");
    let mut connection = new_connection();
    connection.connect().await.unwrap();
}

async fn before_test() {
    static BEFORE_ALL_TESTS: OnceCell<()> = OnceCell::const_new();
    BEFORE_ALL_TESTS.get_or_init(before_all_tests).await;
}

#[tokio::test]
#[serial]
async fn test_add_tag_command() -> io::Result<()> {
    before_test().await;
    // given:
    let mut connection = new_connection();
    let listener_name = "test";
    let tag = "success";
    let commands = [
        Command::new("say running test_add_tag_command"),
        Command::new(logged_command(enable_logging_command())),
        Command::new(named_logged_command(
            listener_name,
            add_tag_command("@s", tag),
        )),
        Command::new(logged_command(reset_logging_command())),
    ];
    let mut events = connection.add_named_listener(listener_name);

    // when:
    connection.execute_commands(commands)?;

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
    before_test().await;
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
    let event = timeout(Duration::from_secs(5), events.next())
        .await?
        .unwrap();
    let output = event.output.parse::<SummonNamedEntityOutput>().unwrap();
    assert_eq!(output.name, name);

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_query_scoreboard_command() -> io::Result<()> {
    before_test().await;
    // given:
    let mut connection = new_connection();
    let listener_name = "test";
    let scoreboard = "minect_test_global";
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
    let event = timeout(Duration::from_secs(5), events.next())
        .await?
        .unwrap();
    let output = event.output.parse::<QueryScoreboardOutput>().unwrap();
    assert_eq!(output.scoreboard, scoreboard);
    assert_eq!(output.score, 42);

    Ok(())
}
