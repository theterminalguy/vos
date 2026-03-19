//! VOS CLI - Main entry point for VOS shell.

use std::io::{self, Write};
use vos_userspace::Shell;

fn main() {
    println!("VOS Shell v0.1.0");
    println!("Type 'help' for available commands, 'exit' to quit.\n");

    let mut shell = Shell::new();

    loop {
        // Display prompt
        print!("vos> ");
        io::stdout().flush().unwrap();

        // Read input
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Error reading input");
            break;
        }

        // Execute command
        match shell.execute(input.trim()) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }

        // Check if should exit
        if shell.should_exit() {
            break;
        }
    }
}
