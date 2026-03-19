//! VOS Debugger - Interactive debugger for VOS programs.
//!
//! Provides step-by-step execution, breakpoints, and state inspection.
//!
//! # Features
//!
//! - **Step Execution**: Execute one instruction at a time
//! - **Breakpoints**: Set breakpoints at specific addresses
//! - **State Inspection**: View registers, memory, and CPU flags
//! - **Continue**: Run until breakpoint or halt
//! - **Disassembly**: View current instruction
//!
//! # Commands
//!
//! ```text
//! step (s)         - Execute one instruction
//! continue (c)     - Run until breakpoint or halt
//! break <addr>     - Set breakpoint at address
//! delete <num>     - Delete breakpoint
//! info registers   - Display all registers
//! info memory <addr> [count] - Display memory
//! disassemble      - Show current instruction
//! quit (q)         - Exit debugger
//! help (h)         - Show help
//! ```

pub mod debugger;

pub use debugger::Debugger;
