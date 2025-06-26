mod pipe;
mod process;

pub type Empty = anyhow::Result<()>;

pub mod prelude {
    pub use super::Empty;
    pub use super::pipe::Pipe;
    pub use super::process::Process;
}

#[cfg(test)]
mod tests;
