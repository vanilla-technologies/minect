use std::{error::Error, io};

pub fn io_invalid_data<E>(error: E) -> io::Error
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::InvalidData, error)
}
