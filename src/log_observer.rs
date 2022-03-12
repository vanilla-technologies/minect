use crate::utils::{io_broken_pipe, io_other};
use notify::{raw_watcher, Op, RawEvent, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Seek, SeekFrom},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        mpsc::{channel, RecvTimeoutError},
        Arc, RwLock,
    },
    thread,
};
use tokio::{
    sync::mpsc::{error::SendError, unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::Duration,
};

pub(crate) struct LogObserver {
    path: PathBuf,
    listeners: Arc<RwLock<HashMap<String, UnboundedSender<LogEvent>>>>,
    listener_vec: Arc<RwLock<Vec<UnboundedSender<LogEvent>>>>,
}

impl LogObserver {
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> LogObserver {
        let path = path.as_ref().to_path_buf();
        let listeners = Arc::new(RwLock::new(HashMap::new()));
        let listener_vec = Arc::new(RwLock::new(Vec::new()));

        let observer = LogObserver {
            path: path.clone(),
            listeners: listeners.clone(),
            listener_vec: listener_vec.clone(),
        };
        thread::spawn(|| {
            observer.observe_log().unwrap(); // TODO panic
        });

        LogObserver {
            path,
            listeners,
            listener_vec,
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
        while Arc::strong_count(&self.listeners) > 1 {
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
            let listeners = self.listener_vec.read().unwrap();
            let mut delete_indexes = Vec::new();
            for (index, listener) in listeners.iter().enumerate() {
                if let Err(SendError(_event)) = listener.send(event.clone()) {
                    delete_indexes.push(index);
                }
            }
            drop(listeners);
            if !delete_indexes.is_empty() {
                let mut listeners = self.listener_vec.write().unwrap();
                for delete_index in delete_indexes {
                    listeners.remove(delete_index);
                }
            }

            let listeners = self.listeners.read().unwrap();
            if let Some(listener) = listeners.get(&event.executor) {
                if let Err(SendError(event)) = listener.send(event) {
                    drop(listeners);
                    let mut listeners = self.listeners.write().unwrap();
                    listeners.remove(&event.executor);
                }
            }
        }
    }

    pub(crate) fn add_general_listener(&mut self) -> UnboundedReceiver<LogEvent> {
        let (sender, receiver) = unbounded_channel();
        self.listener_vec.write().unwrap().push(sender);
        receiver
    }

    pub(crate) fn add_listener(&mut self, name: &str) -> UnboundedReceiver<LogEvent> {
        let (sender, receiver) = unbounded_channel();
        self.listeners
            .write()
            .unwrap()
            .insert(name.to_string(), sender);
        receiver
    }
}

/// A [LogEvent] represents a line in Minecrafts log file that written when a command is executed
/// successfully.
///
/// Here is an example:
/// ```none
/// [13:14:30] [Server thread/INFO]: [executor: message]
/// ```
#[derive(Clone, Debug)]
pub struct LogEvent {
    pub executor: String,
    pub message: String,
    private: (),
}

impl FromStr for LogEvent {
    type Err = ();

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        fn from_string_opt(line: &str) -> Option<LogEvent> {
            const ZERO_TO_NINE: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
            let (executor, message) = line
                .strip_prefix('[')?
                .strip_prefix(ZERO_TO_NINE)?
                .strip_prefix(ZERO_TO_NINE)?
                .strip_prefix(':')?
                .strip_prefix(ZERO_TO_NINE)?
                .strip_prefix(ZERO_TO_NINE)?
                .strip_prefix(':')?
                .strip_prefix(ZERO_TO_NINE)?
                .strip_prefix(ZERO_TO_NINE)?
                .strip_prefix("] [Server thread/INFO]: [")?
                .trim_end()
                .strip_suffix(']')?
                .split_once(": ")?;
            Some(LogEvent {
                executor: executor.to_string(),
                message: message.to_string(),
                private: (),
            })
        }
        from_string_opt(line).ok_or(())
    }
}
