use notify::{raw_watcher, Op, RecursiveMode, Watcher};
use std::{io, path::Path, sync::mpsc::channel};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::{sleep, Duration},
};

fn watch_logfile<P: AsRef<Path>>(path: P, sender: UnboundedSender<()>) -> io::Result<()> {
    let (event_sender, event_reciever) = channel();
    let mut watcher = raw_watcher(event_sender).map_err(other)?;
    watcher
        .watch(path, RecursiveMode::NonRecursive)
        .map_err(other)?;

    loop {
        let event = event_reciever.recv().map_err(broken_pipe)?;
        if let Ok(Op::CREATE) = event.op {
            if let Err(_) = sender.send(()) {
                break Ok(());
            }
        }
    }
}

fn broken_pipe<E>(e: E) -> io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::BrokenPipe, e)
}

fn other<E>(e: E) -> io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::Other, e)
}

async fn read_logfile<P: AsRef<Path>>(
    path: P,
    mut reciever: UnboundedReceiver<()>,
) -> io::Result<()> {
    let mut reader = BufReader::new(File::open(&path).await?);
    loop {
        tokio::select! {
            _ = reciever.recv() => {
                // read old logfile to completion before opening new logfile
                continue_read_file(&mut reader, false).await?;
                reader = BufReader::new(File::open(&path).await?);
            }
            _ = continue_read_file(&mut reader, true) => {}
        };
    }
}

async fn continue_read_file(reader: &mut BufReader<File>, blocking: bool) -> io::Result<()> {
    let mut line = String::new();
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read != 0 {
            process_line(&line);
        } else {
            if blocking {
                sleep(Duration::from_millis(50)).await;
            }
            break Ok(());
        }
    }
}

fn process_line(_line: &String) -> () {
    unimplemented!()
}
