//! VOS Script Language - Compiler and interpreter for vos script.
//!
//! A simple TypeScript/Ruby-like scripting language for VOS.

pub mod token;
pub mod lexer;

pub use token::{Token, TokenKind};
pub use lexer::Lexer;
