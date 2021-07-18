use std::{error::Error, io};

pub(crate) fn io_broken_pipe<E>(error: E) -> io::Error
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::BrokenPipe, error)
}

pub(crate) fn io_invalid_data<E>(error: E) -> io::Error
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::InvalidData, error)
}

pub(crate) fn io_other<E>(error: E) -> io::Error
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::Other, error)
}
