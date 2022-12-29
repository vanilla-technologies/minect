use minect::{
    enable_logging_command, logged_command, named_logged_command, reset_logging_command,
    MinecraftConnection,
};
use serial_test::serial;
use std::{io, time::Duration};
use tokio::time::timeout;

const TEST_WORLD_DIR: &str = env!("TEST_WORLD_DIR");
const TEST_LOG_FILE: &str = env!("TEST_LOG_FILE");

fn new_connection() -> MinecraftConnection {
    MinecraftConnection::builder("test", TEST_WORLD_DIR)
        .log_file(TEST_LOG_FILE)
        .build()
}

#[tokio::test]
#[serial]
async fn test_tag() -> io::Result<()> {
    // given:
    let mut connection = new_connection();
    let name = "test";
    let commands = vec![
        "say running test_tag".to_string(),
        enable_logging_command(),
        named_logged_command(name, "tag @s add success"),
        reset_logging_command(),
    ];
    let mut events = connection.add_listener(name);

    // when:
    connection.inject_commands(commands)?;

    // then:
    let event = timeout(Duration::from_secs(5), events.recv())
        .await?
        .unwrap();
    assert_eq!(event.message, format!("Added tag 'success' to {}", name));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_score_objective() -> io::Result<()> {
    // given:
    let mut connection = new_connection();
    let name = "test";
    let commands = vec![
        "say running test_score_objective".to_string(),
        enable_logging_command(),
        named_logged_command(name, "scoreboard objectives add success dummy"),
        logged_command("scoreboard objectives remove success"),
        reset_logging_command(),
    ];
    let mut events = connection.add_listener(name);

    // when:
    connection.inject_commands(commands)?;

    // then:
    let event = timeout(Duration::from_secs(5), events.recv())
        .await?
        .unwrap();
    assert_eq!(event.message, "Created new objective [success]");

    Ok(())
}
