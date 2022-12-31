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

use crate::{
    utils::{io_broken_pipe, io_other},
    LoadedListener, LogEvent,
};
use notify::{raw_watcher, Op, RawEvent, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::{
        mpsc::{channel, RecvTimeoutError},
        Arc, RwLock,
    },
    thread,
};
use tokio::{
    sync::mpsc::{error::SendError, unbounded_channel, UnboundedSender},
    time::Duration,
};
use tokio_stream::{wrappers::UnboundedReceiverStream, Stream};

pub struct LogObserver {
    path: PathBuf,
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

        let observer = LogObserver {
            path: path.clone(),
            loaded_listeners: loaded_listeners.clone(),
            listeners: listeners.clone(),
            named_listeners: named_listeners.clone(),
        };
        thread::spawn(|| {
            observer.observe_log().unwrap(); // TODO panic
        });

        LogObserver {
            path,
            loaded_listeners,
            listeners,
            named_listeners,
        }
    }

    fn observe_log(self) -> io::Result<()> {
        let (event_sender, event_reciever) = channel();
        let mut watcher = raw_watcher(event_sender).map_err(io_other)?;
        watcher
            .watch(&self.path, RecursiveMode::NonRecursive)
            .map_err(io_other)?;

        let mut reader = BufReader::new(File::open(&self.path)?);
        reader.seek(SeekFrom::End(0))?;

        // Watch log file as long as the other LogFileObserver is not dropped
        while Arc::strong_count(&self.named_listeners) > 1 {
            let event = event_reciever.recv_timeout(Duration::from_millis(50));
            if let Err(RecvTimeoutError::Disconnected) = event {
                return Err(io_broken_pipe(RecvTimeoutError::Disconnected));
            }
            self.continue_to_read_file(&mut reader)?;
            if let Ok(RawEvent {
                op: Ok(Op::CREATE), ..
            }) = event
            {
                reader = BufReader::new(File::open(&self.path)?);
            }
        }

        Ok(())
    }

    fn continue_to_read_file(&self, reader: &mut BufReader<File>) -> io::Result<()> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read != 0 {
                self.process_line(&line);
            } else {
                break Ok(());
            }
        }
    }

    fn process_line(&self, line: &String) {
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

    pub(crate) fn add_loaded_listener(&mut self, listener: LoadedListener) {
        self.loaded_listeners.write().unwrap().push(listener);
    }

    pub fn add_listener(&mut self) -> impl Stream<Item = LogEvent> {
        let (sender, receiver) = unbounded_channel();
        self.listeners.write().unwrap().push(sender);
        UnboundedReceiverStream::new(receiver)
    }

    pub fn add_named_listener(&mut self, name: impl Into<String>) -> impl Stream<Item = LogEvent> {
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
