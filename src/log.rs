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

pub mod observer;

use std::{
    fmt::{self, Display},
    str::FromStr,
};

use crate::json::{create_json_text_component, escape_json};

/// Generates a Minecraft command that schedules the given command to run in such a way that a
/// [LogEvent] is created if logging is enabled at that time (see [enable_logging_command()]).
///
/// Commands injected via [inject_commands](crate::MinecraftConnection::inject_commands) can
/// generate [LogEvent]s themselfs, but commands in functions called from injected commands can't
/// without using [logged_command()].
///
/// To ensure [LogEvent]s are created, the first logged command should be [enable_logging_command()]
/// and the last one should be [reset_logging_command()]:
/// ```no_run
/// # use minect::*;
/// # use minect::log::*;
/// let my_function_body = [
///     logged_command(enable_logging_command()),
///     logged_command(query_scoreboard_command("@p", "my_scoreboard")),
///     logged_command(reset_logging_command()),
/// ].join("\n");
///
/// // Generate datapack containing my_function ...
///
/// # let mut connection = MinecraftConnection::builder("", "").build();
/// connection.inject_commands(["function my_namespace:my_function"])?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// The generated command summons a command block minecart that runs the given command. This may
/// cause a small delay, because command block minecart don't execute very game tick.
///
pub fn logged_command(command: impl Into<String>) -> String {
    LoggedCommandBuilder::new(command).to_string()
}

/// Same as [logged_command()] but also gives the command block minecart a CustomName to allow easy
/// filtering of [LogEvent]s.
pub fn named_logged_command(name: &str, command: impl Into<String>) -> String {
    LoggedCommandBuilder::new(command).name(name).to_string()
}

/// Generates a command that ensures [LogEvent]s are created for all commands until a
/// [reset_logging_command()] is executed.
///
/// This command sets the following three gamerules:
/// 1. `logAdminCommands`: This must be `true` for Minecraft to log the output of commands to the
///    log file.
/// 2. `commandBlockOutput`: This must be `true` for command blocks and command block minecarts to
///    "publish" the outbut of their commands.
/// 3. `sendCommandFeedback`: This is set to `false` to prevent the output to be logged in the chat
///    which would likely annoy players.
///
/// This changes the logging configuration of the world in such a way that a player does not get any
/// output from any command (including commands the player executes). So the original values of the
/// gamerules are stored and can be restored by executing a [reset_logging_command()].
///
/// Be careful not to execute two [enable_logging_command()]s before executing a
/// [reset_logging_command()], as that would overwrite the stored gamrule values, thus preventing
/// restoring the original values.
pub fn enable_logging_command() -> String {
    "function minect:enable_logging".to_string()
}

/// Generates a command that restores the logging gamerules to their values before executing the
/// last [enable_logging_command()].
pub fn reset_logging_command() -> String {
    "function minect:reset_logging".to_string()
}

/// A builder to generate [logged commands](logged_command()).
pub struct LoggedCommandBuilder {
    custom_name: Option<String>,
    command: String,
}

impl LoggedCommandBuilder {
    /// Creates a new builder to generate a [logged](logged_command()) version of the given command.
    pub fn new(command: impl Into<String>) -> LoggedCommandBuilder {
        LoggedCommandBuilder {
            custom_name: None,
            command: command.into(),
        }
    }

    /// Sets the CustomName of the command block minecart to the given JSON text component.
    pub fn custom_name(mut self, custom_name: impl Into<String>) -> LoggedCommandBuilder {
        self.custom_name = Some(custom_name.into());
        self
    }

    /// Sets the CustomName of the command block minecart to the given string.
    pub fn name(self, name: &str) -> LoggedCommandBuilder {
        self.custom_name(create_json_text_component(name))
    }
}

impl Display for LoggedCommandBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "execute at @e[type=area_effect_cloud,tag=minect_connection,limit=1] \
                run summon command_block_minecart ~ ~ ~ {",
        )?;
        if let Some(custom_name) = &self.custom_name {
            write!(f, "\"CustomName\":\"{}\",", escape_json(custom_name))?;
        }
        write!(f, "\"Command\":\"{}\",", self.command)?;
        f.write_str(
            "\
            \"Tags\":[\"minect_impulse\"],\
            \"LastExecution\":1L,\
            \"TrackOutput\":false,\
        }",
        )
    }
}

/// A [LogEvent] is created for every command that is successfully executed and logged.
///
/// Commands are logged if logging is enabled (see [enable_logging_command()]) and the command is
/// executed by a player, command block or command block minecart. Commands in functions are never
/// logged. To work around this use [logged_command()].
///
/// This is what a [LogEvent] looks like in Minecrafts log file:
/// ```none
/// [13:14:30] [Server thread/INFO]: [executor: output]
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogEvent {
    hour: u8,
    minute: u8,
    second: u8,
    /// The name of the player, command block or command block minecart that executed the command.
    pub executor: String,
    /// The output of the command.
    pub output: String,
    _private: (),
}

impl FromStr for LogEvent {
    type Err = ();

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        fn from_str_opt(line: &str) -> Option<LogEvent> {
            let line = line.strip_prefix('[')?;
            let (hour, line) = read_digits(line, 2)?;
            let line = line.strip_prefix(':')?;
            let (minute, line) = read_digits(line, 2)?;
            let line = line.strip_prefix(':')?;
            let (second, line) = read_digits(line, 2)?;
            let line = line.strip_prefix("] [Server thread/INFO]: [")?;
            let line = line.trim_end();
            let line = line.strip_suffix(']')?;
            let (executor, output) = line.split_once(": ")?;

            Some(LogEvent {
                hour,
                minute,
                second,
                executor: executor.to_string(),
                output: output.to_string(),
                _private: (),
            })
        }
        from_str_opt(line).ok_or(())
    }
}

fn read_digits<N: FromStr>(string: &str, len: usize) -> Option<(N, &str)> {
    if string.len() >= len && string[..len].bytes().all(|b| b.is_ascii_digit()) {
        let number = string[..len].parse().ok()?;
        Some((number, &string[len..]))
    } else {
        None
    }
}

impl Display for LogEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}:{}:{}] [Server thread/INFO]: [{}: {}]",
            self.hour, self.minute, self.second, self.executor, self.output
        )
    }
}

/// Generates a Minecraft command that summons an area effect cloud with the given `name`.
///
/// The resulting [LogEvent::output] can be parsed into a [SummonNamedEntityOutput].
///
/// `name` is interpreted as a string, not a JSON text component.
///
/// By using a unique `name` this command can be used inside an `execute if` command to check if
/// some condition is true in Minecraft. A good way to generate a unique `name` is to use a UUID.
///
/// When using a [logged_command()] then [add_tag_command()] is usually a better alternative in
/// terms of performance, because it avoids the overhead of summoning a new entity.
pub fn summon_named_entity_command(name: &str) -> String {
    let custom_name = create_json_text_component(name);
    format!(
        "summon area_effect_cloud ~ ~ ~ {{\"CustomName\":\"{}\"}}",
        escape_json(&custom_name)
    )
}

/// The output of a [summon_named_entity_command()]. This can be parsed from a [LogEvent::output].
///
/// The output has the following format:
/// ```none
/// Summoned new <name>
/// ```
///
/// For example:
/// ```none
/// Summoned new my_name
/// ```
pub struct SummonNamedEntityOutput {
    /// The name of the summoned entity.
    pub name: String,
    _private: (),
}
impl FromStr for SummonNamedEntityOutput {
    type Err = ();

    fn from_str(output: &str) -> Result<Self, Self::Err> {
        fn from_str_opt(output: &str) -> Option<SummonNamedEntityOutput> {
            let name = output.strip_prefix("Summoned new ")?;

            Some(SummonNamedEntityOutput {
                name: name.to_string(),
                _private: (),
            })
        }
        from_str_opt(output).ok_or(())
    }
}
impl Display for SummonNamedEntityOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Summoned new {}", self.name)
    }
}

/// Generates a Minecraft command that adds the given `tag` to the given `entity`.
///
/// The resulting [LogEvent::output] can be parsed into an [AddTagOutput].
///
/// `entity` can be any selector or name.
///
/// For a [logged_command()] that only uses this tag as a means to know when/if the command is
/// executed (for example inside an `execute if` command) it can be useful to add a tag to the `@s`
/// entity. This saves the trouble of removing the tag again, because the command block minecart is
/// killed after the command is executed. Otherwise the tag will likely need to be removed, because
/// adding a tag twice to the same entity fails, thus preventing further [LogEvent]s.
pub fn add_tag_command(entity: impl Display, tag: impl Display) -> String {
    format!("tag {} add {}", entity, tag)
}

/// The output of a [add_tag_command()]. This can be parsed from a [LogEvent::output].
///
/// The output has the following format:
/// ```none
/// Added tag '<tag>' to <entity>
/// ```
///
/// For example:
/// ```none
/// Added tag 'my_tag' to my_entity
/// ```
pub struct AddTagOutput {
    /// The tag that was added.
    pub tag: String,
    /// The CustomName or UUID of the entity the tag was added to.
    pub entity: String,
    _private: (),
}
impl FromStr for AddTagOutput {
    type Err = ();

    fn from_str(output: &str) -> Result<Self, Self::Err> {
        fn from_str_opt(output: &str) -> Option<AddTagOutput> {
            let suffix = output.strip_prefix("Added tag '")?;
            let (tag, entity) = suffix.split_once("' to ")?;

            Some(AddTagOutput {
                tag: tag.to_string(),
                entity: entity.to_string(),
                _private: (),
            })
        }
        from_str_opt(output).ok_or(())
    }
}
impl Display for AddTagOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Added tag '{}' to {}", self.tag, self.entity)
    }
}

/// Generates a Minecraft command that queries the score of `entity` in `scoreboard`.
///
/// The resulting [LogEvent::output] can be parsed into a [QueryScoreboardOutput].
///
/// `entity` can be any selector or name.
pub fn query_scoreboard_command(entity: impl Display, scoreboard: impl Display) -> String {
    format!("scoreboard players add {} {} 0", entity, scoreboard)
}

/// The output of a [query_scoreboard_command()]. This can be parsed from [LogEvent::output].
///
/// The output has the following format:
/// ```none
/// Added 0 to [<scoreboard>] for <entity> (now <score>)
/// ```
///
/// For example:
/// ```none
/// Added 0 to [my_scoreboard] for my_entity (now 42)
/// ```
pub struct QueryScoreboardOutput {
    /// The scoreboard.
    pub scoreboard: String,
    /// The name of the player or UUID of the entity.
    pub entity: String,
    /// The score of the entity.
    pub score: i32,
    _private: (),
}
impl FromStr for QueryScoreboardOutput {
    type Err = ();

    fn from_str(output: &str) -> Result<Self, Self::Err> {
        fn from_str_opt(output: &str) -> Option<QueryScoreboardOutput> {
            let suffix = output.strip_prefix("Added 0 to [")?;
            const FOR: &str = "] for ";
            let index = suffix.find(FOR)?;
            let (scoreboard, suffix) = suffix.split_at(index);
            let suffix = suffix.strip_prefix(FOR)?;

            const NOW: &str = " (now ";
            let index = suffix.rfind(NOW)?;
            let (entity, suffix) = suffix.split_at(index);
            let suffix = suffix.strip_prefix(NOW)?;
            let score = suffix.strip_suffix(')')?;
            let score = score.parse().ok()?;

            Some(QueryScoreboardOutput {
                scoreboard: scoreboard.to_string(),
                entity: entity.to_string(),
                score,
                _private: (),
            })
        }
        from_str_opt(output).ok_or(())
    }
}
impl Display for QueryScoreboardOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Added 0 to [{}] for {} (now {})",
            self.scoreboard, self.entity, self.score
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_to_string() {
        // given:
        let string = "[21:39:40] [Server thread/INFO]: [test: Added tag 'success' to test]";

        // when:
        let actual_event = string.parse::<LogEvent>().unwrap();
        let actual_string = actual_event.to_string();

        // then:
        assert_eq!(actual_event.executor, "test");
        assert_eq!(actual_event.output, "Added tag 'success' to test");
        assert_eq!(actual_string, string);
    }
}
