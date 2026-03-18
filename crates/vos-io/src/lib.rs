//! VOS I/O - Input/Output devices for the Virtual Operating System.
//!
//! This crate implements memory-mapped I/O devices:
//!
//! - **Display**: Text-mode display (80x25 characters)
//! - **Keyboard**: Keyboard input with buffer
//! - **Timer**: Programmable timer for interrupts
//!
//! All devices implement the `Device` trait from `vos-core` and are memory-mapped
//! to specific address ranges in the I/O region (0x80000000+).
//!
//! # Memory Map
//!
//! ```text
//! 0x80000000 - Display (4000 bytes)
//! 0x80002000 - Keyboard (16 bytes)
//! 0x80003000 - Timer (16 bytes)
//! ```
//!
//! # Examples
//!
//! ## Using the Display
//!
//! ```
//! use vos_io::display::Display;
//! use vos_core::Device;
//!
//! let mut display = Display::new();
//!
//! // Write characters
//! display.write_byte(0, b'H').unwrap();
//! display.write_byte(2, b'i').unwrap();
//!
//! // Or use put_char for easier writing
//! display.put_char(b'!');
//! ```
//!
//! ## Using the Keyboard
//!
//! ```
//! use vos_io::keyboard::Keyboard;
//! use vos_core::Device;
//!
//! let mut keyboard = Keyboard::new();
//!
//! // Simulate key press
//! keyboard.push_key(b'A');
//!
//! // Read from device
//! let ch = keyboard.read_byte(0).unwrap();
//! assert_eq!(ch, b'A');
//! ```
//!
//! ## Using the Timer
//!
//! ```
//! use vos_io::timer::Timer;
//! use vos_core::Device;
//!
//! let mut timer = Timer::new();
//!
//! // Set reload value
//! timer.write_word(4, 1000).unwrap();
//!
//! // Enable timer
//! timer.write_byte(8, 0x01).unwrap();
//!
//! // Tick forward
//! timer.tick(500);
//! assert_eq!(timer.counter(), 500);
//! ```

pub mod display;
pub mod keyboard;
pub mod timer;

// Re-export commonly used items
pub use display::{Display, DISPLAY_BASE, DISPLAY_HEIGHT, DISPLAY_SIZE, DISPLAY_WIDTH};
pub use keyboard::{Keyboard, KEYBOARD_BASE};
pub use timer::{Timer, TIMER_BASE};
