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

//! Observing Minecraft's log file.

mod observer;
pub use observer::LogObserver;

use std::{fmt::Display, str::FromStr};

/// A [LogEvent] is created for every command that is successfully executed and logged.
///
/// There are a few preconditions for commands to write their output to the log file. They are
/// documented on the [command](crate::command) module in detail. That module also contains
/// functions to generate common commands that produce useful [LogEvent]s.
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
            "[{:02}:{:02}:{:02}] [Server thread/INFO]: [{}: {}]",
            self.hour, self.minute, self.second, self.executor, self.output
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_to_string() {
        // given:
        let string = "[21:05:40] [Server thread/INFO]: [test: Added tag 'success' to test]";

        // when:
        let actual_event = string.parse::<LogEvent>().unwrap();
        let actual_string = actual_event.to_string();

        // then:
        assert_eq!(actual_event.executor, "test");
        assert_eq!(actual_event.output, "Added tag 'success' to test");
        assert_eq!(actual_string, string);
    }
}
