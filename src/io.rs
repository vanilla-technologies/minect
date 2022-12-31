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

use std::{
    fmt::Display,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct IoErrorAtPath {
    pub message: String,
    pub path: PathBuf,
    pub cause: io::Error,
}
impl Display for IoErrorAtPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}: {}",
            self.message,
            self.path.display(),
            self.cause
        )
    }
}
impl IoErrorAtPath {
    pub fn new(
        message: impl Into<String>,
        path: impl Into<PathBuf>,
        cause: io::Error,
    ) -> IoErrorAtPath {
        IoErrorAtPath {
            message: message.into(),
            path: path.into(),
            cause,
        }
    }

    pub fn mapper(
        message: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> impl FnOnce(io::Error) -> IoErrorAtPath {
        |cause| IoErrorAtPath::new(message, path, cause)
    }
}
pub(crate) fn io_error(
    message: impl Into<String>,
    path: impl Into<PathBuf>,
) -> impl FnOnce(io::Error) -> IoErrorAtPath {
    IoErrorAtPath::mapper(message, path)
}

impl From<IoErrorAtPath> for io::Error {
    fn from(value: IoErrorAtPath) -> io::Error {
        io::Error::new(value.cause.kind(), value.to_string())
    }
}

pub(crate) fn create(path: impl AsRef<Path>) -> Result<File, IoErrorAtPath> {
    File::create(&path).map_err(io_error("Failed to create file", path.as_ref()))
}

pub(crate) fn write(path: impl AsRef<Path>, contents: &str) -> Result<(), IoErrorAtPath> {
    if let Some(parent) = path.as_ref().parent() {
        create_dir_all(parent)?;
    }
    fs::write(&path, contents).map_err(io_error("Failed to create file", path.as_ref()))
}

pub(crate) fn create_dir_all(path: impl AsRef<Path>) -> Result<(), IoErrorAtPath> {
    fs::create_dir_all(&path).map_err(io_error("Failed to create directory", path.as_ref()))?;
    Ok(())
}

pub(crate) fn remove_dir_all(path: impl AsRef<Path>) -> Result<(), IoErrorAtPath> {
    fs::remove_dir_all(&path).map_err(io_error("Failed to remove directory", path.as_ref()))?;
    Ok(())
}

pub(crate) fn remove_file(path: impl AsRef<Path>) -> Result<(), IoErrorAtPath> {
    fs::remove_file(&path).map_err(io_error("Failed to remove file", path.as_ref()))?;
    Ok(())
}
