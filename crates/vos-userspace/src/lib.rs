//! VOS Userspace - Shell and user programs.
//!
//! Provides user-facing components including an interactive shell.

pub mod shell;
pub mod programs;

pub use shell::Shell;
pub use programs::{Browser, HtmlParser, Element};
