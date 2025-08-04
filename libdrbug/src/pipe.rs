use std::fs::File;
use std::io::{
    ErrorKind,
    Read,
    Write,
};

use nix::fcntl::OFlag;
use nix::unistd::pipe2;

use crate::{
    DrbugError,
    DrbugResult,
    syscall_error,
};

pub struct Pipe {
    reader: Option<File>,
    writer: Option<File>,
}

impl Pipe {
    #[cfg(test)]
    pub fn new() -> DrbugResult<Self> {
        Self::make_pipe(false)
    }

    pub fn new_exec_safe() -> DrbugResult<Self> {
        Self::make_pipe(true)
    }

    pub fn close_reader(&mut self) {
        self.take_reader(); // take it out of the option and drop it, closing the file
    }

    pub fn close_writer(&mut self) {
        self.take_writer(); // take it out of the option and drop it, closing the file
    }

    pub fn read(&mut self) -> DrbugResult<Vec<u8>> {
        let mut buf = [0; 1024];
        let n = self.reader.as_ref().ok_or(DrbugError::PipeClosed)?.read(&mut buf)?;
        Ok(buf[..n].to_owned())
    }

    pub fn take_reader(&mut self) -> Option<File> {
        self.reader.take()
    }

    pub fn take_writer(&mut self) -> Option<File> {
        self.writer.take()
    }

    fn make_pipe(close_on_exec: bool) -> DrbugResult<Self> {
        let flags = if close_on_exec { OFlag::O_CLOEXEC } else { OFlag::empty() };
        let (read_fd, write_fd) = syscall_error!(pipe2(flags))?;
        Ok(Pipe {
            reader: Some(read_fd.into()),
            writer: Some(write_fd.into()),
        })
    }
}

impl Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer
            .as_ref()
            .ok_or(std::io::Error::new(ErrorKind::BrokenPipe, "writer closed"))?
            .write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer
            .as_ref()
            .ok_or(std::io::Error::new(ErrorKind::BrokenPipe, "writer closed"))?
            .flush()
    }
}
