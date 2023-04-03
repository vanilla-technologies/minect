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

//! Functions for generating commands that generate [LogEvent](crate::log::LogEvent)s.

use crate::json::{create_json_text_component, escape_json};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

/// Generates a command that ensures [LogEvent](crate::log::LogEvent)s are created for all commands
/// until a [reset_logging_command] is executed. These two commands are executed automatically by
/// [execute_commands](crate::MinecraftConnection::execute_commands) if
/// [enable_logging_automatically](crate::MinecraftConnectionBuilder::enable_logging_automatically)
/// is `true` (which is the default).
///
/// This command sets the following three gamerules:
/// 1. `logAdminCommands`: This must be `true` for Minecraft to write the output of commands to the
///    log file.
/// 2. `commandBlockOutput`: This must be `true` for command blocks and command block minecarts to
///    broadcast the output of their commands.
/// 3. `sendCommandFeedback`: This is set to `false` to prevent the output to to also be written to
///    the chat which would likely annoy players.
///
/// This changes the logging configuration of the world in such a way that a player does not get any
/// output from any command (including commands the player executes). So the original values of the
/// gamerules are stored and can be restored by executing a [reset_logging_command].
///
/// After executing multiple [enable_logging_command]s, the same number of [reset_logging_command]s
/// has to be executed to reset logging.
pub fn enable_logging_command() -> String {
    "function minect:enable_logging".to_string()
}

/// Generates a command that restores the logging gamerules to their values before executing the
/// last [enable_logging_command].
pub fn reset_logging_command() -> String {
    "function minect:reset_logging".to_string()
}

pub fn logged_block_commands(command: &str) -> [String; 2] {
    [
        prepare_logged_block_command(),
        logged_block_command(command),
    ]
}

pub fn named_logged_block_commands(name: &str, command: &str) -> [String; 2] {
    [
        prepare_logged_block_command(),
        named_logged_block_command(name, command),
    ]
}

pub fn json_named_logged_block_commands(name: &str, command: &str) -> [String; 2] {
    [
        prepare_logged_block_command(),
        json_named_logged_block_command(name, command),
    ]
}

pub fn prepare_logged_block_command() -> String {
    "function minect:prepare_logged_block".to_string()
}

const EXECUTE_AT_CURSOR: &str = "execute at @e[type=area_effect_cloud,tag=minect_cursor] run";

pub fn logged_block_command(command: impl AsRef<str>) -> String {
    format!(
        "{} data modify block ~ ~ ~ Command set value \"{}\"",
        EXECUTE_AT_CURSOR,
        escape_json(command.as_ref()),
    )
}

pub fn named_logged_block_command(name: impl AsRef<str>, command: impl AsRef<str>) -> String {
    json_named_logged_block_command(&create_json_text_component(name.as_ref()), command)
}

pub fn json_named_logged_block_command(name: impl AsRef<str>, command: impl AsRef<str>) -> String {
    format!(
        "{} data modify block ~ ~ ~ {{}} merge value {{CustomName:\"{}\",Command:\"{}\"}}",
        EXECUTE_AT_CURSOR,
        escape_json(name.as_ref()),
        escape_json(command.as_ref()),
    )
}

/// Generates a Minecraft command that schedules the given command to run in such a way that a
/// [LogEvent](crate::log::LogEvent) is created if logging is enabled at that time (see
/// [enable_logging_command]).
///
/// Commands executed via [execute_commands](crate::MinecraftConnection::execute_commands) can
/// generate [LogEvent](crate::log::LogEvent)s themselfs, but commands in functions called from
/// these commands can't without using [logged_command].
///
/// To ensure [LogEvent](crate::log::LogEvent)s are created, the first logged command should be
/// [enable_logging_command] and the last one should be [reset_logging_command]:
/// ```no_run
/// # use minect::*;
/// # use minect::command::*;
/// let my_function = [
///     logged_command(enable_logging_command()),
///     logged_command(query_scoreboard_command("@p", "my_scoreboard")),
///     logged_command(reset_logging_command()),
/// ].join("\n");
///
/// // Generate datapack containing my_function ...
///
/// // Call my_function (could also be done in Minecraft)
/// # let mut connection = MinecraftConnection::builder("", "").build();
/// connection.execute_commands([Command::new("function my_namespace:my_function")])?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// The generated command summons a command block minecart that runs the given command. This may
/// cause a small delay, because command block minecart don't execute very game tick.
///
pub fn logged_cart_command(command: impl AsRef<str>) -> String {
    build_logged_cart_command(None, command.as_ref())
}

/// Same as [logged_command] but also gives the command block minecart a custom name to allow easy
/// filtering of [LogEvent](crate::log::LogEvent)s with
/// [MinecraftConnection::add_named_listener](crate::MinecraftConnection::add_named_listener) or
/// [LogObserver::add_named_listener](crate::log::LogObserver::add_named_listener).
pub fn named_logged_cart_command(name: impl AsRef<str>, command: impl AsRef<str>) -> String {
    json_named_logged_cart_command(&create_json_text_component(name.as_ref()), command)
}

pub fn json_named_logged_cart_command(name: impl AsRef<str>, command: impl AsRef<str>) -> String {
    build_logged_cart_command(Some(name.as_ref()), command.as_ref())
}

fn build_logged_cart_command(name: Option<&str>, command: &str) -> String {
    let custom_name_entry = if let Some(name) = name {
        format!("\"CustomName\":\"{}\",", escape_json(name))
    } else {
        "".to_string()
    };

    format!(
        "execute at @e[type=area_effect_cloud,tag=minect_connection,limit=1] run \
        summon command_block_minecart ~ ~ ~ {{\
            {}\
            \"Command\":\"{}\",\
            \"Tags\":[\"minect_impulse\"],\
            \"LastExecution\":1L,\
            \"TrackOutput\":false,\
        }}",
        custom_name_entry,
        escape_json(command),
    )
}

/// Generates a Minecraft command that summons an area effect cloud with the given `name`.
///
/// The resulting [LogEvent::output](crate::log::LogEvent::output) can be parsed into a
/// [SummonNamedEntityOutput].
///
/// `name` is interpreted as a string, not a JSON text component.
///
/// By using a unique `name` this command can be used inside an `execute if` command to check if
/// some condition is true in Minecraft. A good way to generate a unique `name` is to use a UUID.
///
/// When using [logged_command]s, [add_tag_command] is usually a better alternative in terms of
/// performance, because it avoids the overhead of summoning a new entity.
pub fn summon_named_entity_command(name: &str) -> String {
    let custom_name = create_json_text_component(name);
    format!(
        "summon area_effect_cloud ~ ~ ~ {{\"CustomName\":\"{}\"}}",
        escape_json(&custom_name)
    )
}

/// The output of a [summon_named_entity_command]. This can be parsed from a
/// [LogEvent::output](crate::log::LogEvent::output).
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
#[derive(Clone, Debug, Eq, PartialEq)]
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
/// The resulting [LogEvent::output](crate::log::LogEvent::output) can be parsed into an
/// [AddTagOutput].
///
/// `entity` can be any selector or name.
///
/// For a [logged_command] that only uses this tag as a means to know when/if the command is
/// executed (for example inside an `execute if` command) it can be useful to add a tag to the `@s`
/// entity. This saves the trouble of removing the tag again, because the command block minecart is
/// killed after the command is executed. Otherwise the tag will likely need to be removed, because
/// adding a tag twice to the same entity fails, thus preventing further
/// [LogEvent](crate::log::LogEvent)s.
pub fn add_tag_command(entity: impl Display, tag: impl Display) -> String {
    format!("tag {} add {}", entity, tag)
}

/// The output of an [add_tag_command]. This can be parsed from a
/// [LogEvent::output](crate::log::LogEvent::output).
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddTagOutput {
    /// The tag that was added.
    pub tag: String,
    /// The custom name or UUID of the entity the tag was added to.
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
/// The resulting [LogEvent::output](crate::log::LogEvent::output) can be parsed into a
/// [QueryScoreboardOutput].
///
/// `entity` can be any selector or name.
pub fn query_scoreboard_command(entity: impl Display, scoreboard: impl Display) -> String {
    format!("scoreboard players add {} {} 0", entity, scoreboard)
}

/// The output of a [query_scoreboard_command]. This can be parsed from
/// [LogEvent::output](crate::log::LogEvent::output).
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
#[derive(Clone, Debug, Eq, PartialEq)]
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
mod tests;
