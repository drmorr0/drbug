mod process;

pub type Empty = anyhow::Result<()>;

pub mod prelude {
    pub use super::Empty;
    pub use super::process::Process;
}
