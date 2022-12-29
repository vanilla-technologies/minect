use minect::{LoggedCommand, MinecraftConnection, MinecraftConnectionBuilder};
use serial_test::serial;
use std::{io, time::Duration};
use tokio::time::timeout;

const TEST_WORLD_DIR: &str = env!("TEST_WORLD_DIR");
const TEST_LOG_FILE: &str = env!("TEST_LOG_FILE");

fn new_connection() -> MinecraftConnection {
    MinecraftConnectionBuilder::from_ref("test", TEST_WORLD_DIR)
        .log_file(TEST_LOG_FILE.into())
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
        LoggedCommand::from_str("function minect:enable_logging").to_string(),
        LoggedCommand::builder("tag @s add success".to_string())
            .name(name)
            .build()
            .to_string(),
        LoggedCommand::from_str("function minect:reset_logging").to_string(),
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
        LoggedCommand::from_str("function minect:enable_logging").to_string(),
        LoggedCommand::builder("scoreboard objectives add success dummy".to_string())
            .name(name)
            .build()
            .to_string(),
        LoggedCommand::from_str("scoreboard objectives remove success").to_string(),
        LoggedCommand::from_str("function minect:reset_logging").to_string(),
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
