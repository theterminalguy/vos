//! Interactive debugger implementation.

use std::collections::HashSet;
use vos_core::{Address, Inspectable, Result};
use vos_cpu::instruction::Instruction;
use vos_hardware::VirtualMachine;

/// Breakpoint information.
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Breakpoint number
    pub number: usize,
    /// Address of breakpoint
    pub address: Address,
    /// Whether breakpoint is enabled
    pub enabled: bool,
}

/// Interactive debugger for VOS programs.
///
/// # Examples
///
/// ```no_run
/// use vos_debugger::Debugger;
/// use vos_hardware::VirtualMachine;
///
/// let mut vm = VirtualMachine::new(1024 * 1024);
/// // Load program...
///
/// let mut debugger = Debugger::new(vm);
/// // debugger.run(); // Start interactive session
/// ```
pub struct Debugger {
    /// Virtual machine being debugged
    vm: VirtualMachine,
    /// Breakpoints
    breakpoints: Vec<Breakpoint>,
    /// Next breakpoint number
    next_breakpoint_num: usize,
    /// Whether debugger is running
    running: bool,
}

impl Debugger {
    /// Creates a new debugger with the given virtual machine.
    pub fn new(vm: VirtualMachine) -> Self {
        Self {
            vm,
            breakpoints: Vec::new(),
            next_breakpoint_num: 1,
            running: true,
        }
    }

    /// Returns a reference to the virtual machine.
    pub fn vm(&self) -> &VirtualMachine {
        &self.vm
    }

    /// Returns a mutable reference to the virtual machine.
    pub fn vm_mut(&mut self) -> &mut VirtualMachine {
        &mut self.vm
    }

    /// Executes one instruction (step).
    pub fn step(&mut self) -> Result<bool> {
        self.vm.step()
    }

    /// Continues execution until breakpoint or halt.
    pub fn continue_exec(&mut self) -> Result<()> {
        let breakpoint_addrs: HashSet<Address> = self
            .breakpoints
            .iter()
            .filter(|bp| bp.enabled)
            .map(|bp| bp.address)
            .collect();

        loop {
            let pc = self.vm.cpu().pc();

            // Check for breakpoint
            if breakpoint_addrs.contains(&pc) {
                println!("Breakpoint at 0x{:08X}", pc);
                break;
            }

            // Step
            let should_continue = self.vm.step()?;
            if !should_continue {
                println!("Program halted");
                break;
            }
        }

        Ok(())
    }

    /// Sets a breakpoint at the given address.
    pub fn set_breakpoint(&mut self, address: Address) {
        let breakpoint = Breakpoint {
            number: self.next_breakpoint_num,
            address,
            enabled: true,
        };

        println!("Breakpoint {} at 0x{:08X}", breakpoint.number, address);

        self.breakpoints.push(breakpoint);
        self.next_breakpoint_num += 1;
    }

    /// Deletes a breakpoint by number.
    pub fn delete_breakpoint(&mut self, number: usize) -> bool {
        if let Some(pos) = self.breakpoints.iter().position(|bp| bp.number == number) {
            let bp = self.breakpoints.remove(pos);
            println!("Deleted breakpoint {} at 0x{:08X}", bp.number, bp.address);
            true
        } else {
            println!("No breakpoint number {}", number);
            false
        }
    }

    /// Lists all breakpoints.
    pub fn list_breakpoints(&self) {
        if self.breakpoints.is_empty() {
            println!("No breakpoints set");
            return;
        }

        println!("Num  Address      Enabled");
        println!("---  ----------   -------");
        for bp in &self.breakpoints {
            println!(
                "{:<3}  0x{:08X}   {}",
                bp.number,
                bp.address,
                if bp.enabled { "yes" } else { "no" }
            );
        }
    }

    /// Displays register information.
    pub fn info_registers(&self) {
        println!("\n=== Registers ===");
        for i in 0..16 {
            let value = self.vm.cpu().registers.read(i);
            println!("R{:<2} = 0x{:08X} ({})", i, value, value);
        }

        println!("\nPC  = 0x{:08X}", self.vm.cpu().pc());

        let flags = self.vm.cpu().registers.flags;
        println!("\n=== Flags ===");
        println!("Zero     (Z): {}", flags.zero);
        println!("Negative (N): {}", flags.negative);
        println!("Carry    (C): {}", flags.carry);
        println!("Overflow (V): {}", flags.overflow);
    }

    /// Displays memory at the given address.
    pub fn info_memory(&mut self, address: Address, count: usize) {
        println!("\n=== Memory at 0x{:08X} ===", address);

        for i in 0..count {
            let addr = address + (i as u32 * 4);
            match self.vm.memory_mut().read_word(addr) {
                Ok(word) => {
                    println!(
                        "0x{:08X}: 0x{:08X}  {:3} {:3} {:3} {:3}",
                        addr,
                        word,
                        word & 0xFF,
                        (word >> 8) & 0xFF,
                        (word >> 16) & 0xFF,
                        (word >> 24) & 0xFF
                    );
                }
                Err(e) => {
                    println!("0x{:08X}: Error - {}", addr, e);
                    break;
                }
            }
        }
    }

    /// Disassembles the current instruction.
    pub fn disassemble(&mut self) -> Result<()> {
        let pc = self.vm.cpu().pc();
        let word = self.vm.memory_mut().read_word(pc)?;

        match Instruction::decode(word) {
            Ok(instr) => {
                println!(
                    "0x{:08X}: {} (0x{:08X})",
                    pc,
                    instr.disassemble(),
                    word
                );
            }
            Err(e) => {
                println!("0x{:08X}: Invalid instruction - {} (0x{:08X})", pc, e, word);
            }
        }

        Ok(())
    }

    /// Displays help information.
    pub fn show_help(&self) {
        println!("\nVOS Debugger Commands:");
        println!("  step, s                  - Execute one instruction");
        println!("  continue, c              - Run until breakpoint or halt");
        println!("  break <addr>             - Set breakpoint at address (hex)");
        println!("  delete <num>             - Delete breakpoint by number");
        println!("  list                     - List all breakpoints");
        println!("  info registers, info reg - Display all registers");
        println!("  info memory <addr> [count] - Display memory (default count=8)");
        println!("  disassemble, disas       - Show current instruction");
        println!("  vm                       - Show virtual machine state");
        println!("  quit, q                  - Exit debugger");
        println!("  help, h, ?               - Show this help\n");
    }

    /// Displays virtual machine state.
    pub fn show_vm_state(&self) {
        println!("\n{}", self.vm.inspect());
    }

    /// Returns whether the debugger is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stops the debugger.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vos_cpu::instruction::Opcode;

    fn create_test_vm() -> VirtualMachine {
        let mut vm = VirtualMachine::new(1024);

        // Load a simple program: ADDI R1, R0, 42; HALT
        let addi = Instruction::IType {
            opcode: Opcode::ADDI,
            rt: 1,
            rs: 0,
            immediate: 42,
        };
        let halt = Instruction::IType {
            opcode: Opcode::HALT,
            rt: 0,
            rs: 0,
            immediate: 0,
        };

        vm.load_program(0, &addi.encode().to_le_bytes()).unwrap();
        vm.load_program(4, &halt.encode().to_le_bytes()).unwrap();
        vm.cpu_mut().set_pc(0);

        vm
    }

    #[test]
    fn test_debugger_creation() {
        let vm = create_test_vm();
        let debugger = Debugger::new(vm);

        assert!(debugger.is_running());
        assert_eq!(debugger.breakpoints.len(), 0);
    }

    #[test]
    fn test_step() {
        let vm = create_test_vm();
        let mut debugger = Debugger::new(vm);

        // Step once (execute ADDI)
        let should_continue = debugger.step().unwrap();
        assert!(should_continue);

        // Check that R1 = 42
        assert_eq!(debugger.vm().cpu().registers.read(1), 42);

        // PC should have advanced
        assert_eq!(debugger.vm().cpu().pc(), 4);
    }

    #[test]
    fn test_breakpoint() {
        let vm = create_test_vm();
        let mut debugger = Debugger::new(vm);

        debugger.set_breakpoint(4);
        assert_eq!(debugger.breakpoints.len(), 1);
        assert_eq!(debugger.breakpoints[0].address, 4);
    }

    #[test]
    fn test_delete_breakpoint() {
        let vm = create_test_vm();
        let mut debugger = Debugger::new(vm);

        debugger.set_breakpoint(4);
        let bp_num = debugger.breakpoints[0].number;

        assert!(debugger.delete_breakpoint(bp_num));
        assert_eq!(debugger.breakpoints.len(), 0);
    }

    #[test]
    fn test_continue_to_breakpoint() {
        let vm = create_test_vm();
        let mut debugger = Debugger::new(vm);

        debugger.set_breakpoint(4); // Breakpoint at HALT
        debugger.continue_exec().unwrap();

        // Should have stopped at breakpoint
        assert_eq!(debugger.vm().cpu().pc(), 4);
    }

    #[test]
    fn test_continue_to_halt() {
        let vm = create_test_vm();
        let mut debugger = Debugger::new(vm);

        debugger.continue_exec().unwrap();

        // Should have halted
        assert!(debugger.vm().cpu().is_halted());
    }
}
