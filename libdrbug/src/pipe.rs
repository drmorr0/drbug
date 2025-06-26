use std::fs::File;
use std::io::{
    ErrorKind,
    Read,
    Write,
};

use anyhow::anyhow;
use nix::fcntl::OFlag;
use nix::unistd::pipe2;

pub struct Pipe {
    reader: Option<File>,
    writer: Option<File>,
}

impl Pipe {
    pub fn new() -> anyhow::Result<Self> {
        Self::make_pipe(false)
    }

    pub fn new_exec_safe() -> anyhow::Result<Self> {
        Self::make_pipe(true)
    }

    pub fn close_reader(&mut self) {
        self.reader.take(); // take it out of the option and drop it, closing the file
    }

    pub fn close_writer(&mut self) {
        self.writer.take(); // take it out of the option and drop it, closing the file
    }

    pub fn read(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut buf = [0; 1024];
        let n = self.reader.as_ref().ok_or(anyhow!("reader closed"))?.read(&mut buf)?;
        Ok(buf[..n].to_owned())
    }

    fn make_pipe(close_on_exec: bool) -> anyhow::Result<Self> {
        let flags = if close_on_exec { OFlag::O_CLOEXEC } else { OFlag::empty() };
        let (read_fd, write_fd) = pipe2(flags)?;
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
