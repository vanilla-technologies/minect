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

use crate::{LoadedListener, LogEvent};
use encoding_rs::Encoding;
use log::trace;
use notify::{event::ModifyKind, recommended_watcher, EventKind, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::{
        mpsc::{channel, RecvTimeoutError},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};
use tokio::sync::mpsc::{error::SendError, unbounded_channel, UnboundedSender};
use tokio_stream::{wrappers::UnboundedReceiverStream, Stream};

pub struct LogObserver {
    loaded_listeners: Arc<RwLock<Vec<LoadedListener>>>,
    listeners: Arc<RwLock<Vec<UnboundedSender<LogEvent>>>>,
    named_listeners: Arc<RwLock<HashMap<String, Vec<UnboundedSender<LogEvent>>>>>,
}

impl LogObserver {
    pub fn new<P: AsRef<Path>>(path: P) -> LogObserver {
        let path = path.as_ref().to_path_buf();
        let listeners = Arc::new(RwLock::new(Vec::new()));
        let named_listeners = Arc::new(RwLock::new(HashMap::new()));
        let loaded_listeners = Arc::new(RwLock::new(Vec::new()));

        let backend = LogObserverBackend {
            path,
            loaded_listeners: loaded_listeners.clone(),
            listeners: listeners.clone(),
            named_listeners: named_listeners.clone(),
        };
        thread::spawn(|| backend.observe_log());

        LogObserver {
            loaded_listeners,
            listeners,
            named_listeners,
        }
    }

    pub(crate) fn add_loaded_listener(&self, listener: LoadedListener) {
        self.loaded_listeners.write().unwrap().push(listener);
    }

    pub fn add_listener(&self) -> impl Stream<Item = LogEvent> {
        let (sender, receiver) = unbounded_channel();
        self.listeners.write().unwrap().push(sender);
        UnboundedReceiverStream::new(receiver)
    }

    pub fn add_named_listener(&self, name: impl Into<String>) -> impl Stream<Item = LogEvent> {
        let (sender, receiver) = unbounded_channel();
        self.named_listeners
            .write()
            .unwrap()
            .entry(name.into())
            .or_default()
            .push(sender);
        UnboundedReceiverStream::new(receiver)
    }
}

#[cfg(target_os = "windows")]
static ENCODING: &'static Encoding = encoding_rs::WINDOWS_1252;
#[cfg(not(target_os = "windows"))]
static ENCODING: &'static Encoding = encoding_rs::UTF_8;

struct LogObserverBackend {
    path: PathBuf,
    loaded_listeners: Arc<RwLock<Vec<LoadedListener>>>,
    listeners: Arc<RwLock<Vec<UnboundedSender<LogEvent>>>>,
    named_listeners: Arc<RwLock<HashMap<String, Vec<UnboundedSender<LogEvent>>>>>,
}
impl LogObserverBackend {
    fn observe_log(self) {
        let (event_sender, event_reciever) = channel();
        let mut watcher = recommended_watcher(event_sender).unwrap(); // may panic
        let watch_path = self.path.parent().unwrap_or(&self.path);
        watcher.watch(watch_path, RecursiveMode::Recursive).unwrap(); // may panic

        let mut file = File::open(&self.path).unwrap(); // may panic
        file.seek(SeekFrom::End(0)).unwrap(); // may panic
        let mut reader = BufReader::new(file);
        self.continue_to_read_file(&mut reader);

        // Watch log file as long as the LogFileObserver is not dropped
        while Arc::strong_count(&self.listeners) > 1 {
            // On Windows we don't get any modify events, so we check for changes at least once per game tick
            match event_reciever.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok(event)) if event.paths.contains(&self.path) => match event.kind {
                    EventKind::Create(_) => self.update_reader(&mut reader),
                    EventKind::Modify(ModifyKind::Data(_)) => {
                        self.continue_to_read_file(&mut reader)
                    }
                    _ => {}
                },
                Err(RecvTimeoutError::Timeout) => self.continue_to_read_file(&mut reader),
                Err(RecvTimeoutError::Disconnected) => panic!("File watcher thread crashed!"),
                _ => {}
            }
        }
        trace!("Shutting down LogObserverBackend");
    }

    fn update_reader(&self, reader: &mut BufReader<File>) {
        self.continue_to_read_file(reader);
        if let Ok(file) = File::open(&self.path) {
            trace!("Detected file change");
            *reader = BufReader::new(file);
        }
    }

    fn continue_to_read_file(&self, reader: &mut impl BufRead) {
        let mut buffer = Vec::new();
        loop {
            buffer.clear();
            let bytes_read = reader.read_until(b'\n', &mut buffer).unwrap(); // may panic
            if bytes_read != 0 {
                let (line, _) = ENCODING.decode_without_bom_handling(&buffer);
                self.process_line(&line);
            } else {
                break;
            }
        }
    }

    fn process_line(&self, line: &str) {
        if let Some(event) = line.parse::<LogEvent>().ok() {
            self.send_event_to_loaded_listeners(&event);
            self.send_event_to_listeners(&event);
            self.send_event_to_named_listeners(event);
        }
    }

    fn send_event_to_loaded_listeners(&self, event: &LogEvent) {
        let loaded_listeners = self.loaded_listeners.read().unwrap();
        for loaded_listener in loaded_listeners.iter() {
            loaded_listener.on_event(event.clone())
        }
    }

    fn send_event_to_listeners(&self, event: &LogEvent) {
        let indexes_to_delete = {
            let listeners = self.listeners.read().unwrap();
            send_event_to_listeners(event, listeners.iter())
        };
        if !indexes_to_delete.is_empty() {
            let mut listeners = self.listeners.write().unwrap();
            delete_indexes(&mut listeners, indexes_to_delete);
        }
    }

    fn send_event_to_named_listeners(&self, event: LogEvent) {
        let indexes_to_delete = {
            let named_listeners = self.named_listeners.read().unwrap();
            if let Some(named_listeners) = named_listeners.get(&event.executor) {
                send_event_to_listeners(&event, named_listeners)
            } else {
                Vec::new()
            }
        };
        if !indexes_to_delete.is_empty() {
            let mut named_listeners = self.named_listeners.write().unwrap();
            if let Some(listeners) = named_listeners.get_mut(&event.executor) {
                if indexes_to_delete.len() == listeners.len() {
                    named_listeners.remove(&event.executor);
                } else {
                    delete_indexes(listeners, indexes_to_delete);
                }
            }
        }
    }
}

fn send_event_to_listeners<'l>(
    event: &LogEvent,
    listeners: impl IntoIterator<Item = &'l UnboundedSender<LogEvent>>,
) -> Vec<usize> {
    let mut indexes_to_delete = Vec::new();
    for (index, listener) in listeners.into_iter().enumerate() {
        if let Err(SendError(_event)) = listener.send(event.clone()) {
            indexes_to_delete.push(index);
        }
    }
    indexes_to_delete
}

fn delete_indexes<E>(listeners: &mut Vec<E>, indexes_to_delete: Vec<usize>) {
    // Back to front to avoid index shifting
    for index in indexes_to_delete.into_iter().rev() {
        listeners.remove(index);
    }
}
