//! Virtual Machine implementation.
//!
//! Integrates CPU, Memory, and I/O devices into a complete computer system.

use vos_core::{Address, Byte, Device, Executable, Inspectable, Result, Word};
use vos_cpu::Cpu;
use vos_io::{Display, Keyboard, Timer};
use vos_memory::Memory;

/// Virtual Machine.
///
/// A complete computer system with CPU, Memory, and I/O devices.
///
/// # Architecture
///
/// - **CPU**: 32-bit RISC processor with 16 registers
/// - **Memory**: Configurable RAM with MMU and paging
/// - **Display**: 80x25 text-mode display
/// - **Keyboard**: Input buffer with interrupts
/// - **Timer**: Programmable countdown timer
///
/// # Examples
///
/// ```
/// use vos_hardware::machine::VirtualMachine;
/// use vos_cpu::instruction::{Instruction, Opcode};
///
/// // Create a VM with 1MB of memory
/// let mut vm = VirtualMachine::new(1024 * 1024);
///
/// // Create a HALT instruction
/// let halt = Instruction::IType {
///     opcode: Opcode::HALT,
///     rt: 0,
///     rs: 0,
///     immediate: 0,
/// };
///
/// // Encode and load program
/// let halt_bytes = halt.encode().to_le_bytes();
/// vm.load_program(0x1000, &halt_bytes).unwrap();
///
/// // Set PC and run
/// vm.cpu_mut().set_pc(0x1000);
/// vm.run().unwrap();
/// ```
#[derive(Debug)]
pub struct VirtualMachine {
    /// CPU
    cpu: Cpu,

    /// Memory
    memory: Memory,

    /// Display device
    display: Display,

    /// Keyboard device
    keyboard: Keyboard,

    /// Timer device
    timer: Timer,

    /// Total cycles executed
    cycles: u64,
}

impl VirtualMachine {
    /// Creates a new virtual machine with the specified memory size.
    ///
    /// # Parameters
    ///
    /// - `memory_size`: Size of RAM in bytes (default 16MB recommended)
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_hardware::machine::VirtualMachine;
    ///
    /// let vm = VirtualMachine::new(16 * 1024 * 1024); // 16MB
    /// ```
    pub fn new(memory_size: usize) -> Self {
        Self {
            cpu: Cpu::new(),
            memory: Memory::new(memory_size),
            display: Display::new(),
            keyboard: Keyboard::new(),
            timer: Timer::new(),
            cycles: 0,
        }
    }

    /// Returns a reference to the CPU.
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// Returns a mutable reference to the CPU.
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    /// Returns a reference to memory.
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    /// Returns a mutable reference to memory.
    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    /// Returns a reference to the display.
    pub fn display(&self) -> &Display {
        &self.display
    }

    /// Returns a mutable reference to the display.
    pub fn display_mut(&mut self) -> &mut Display {
        &mut self.display
    }

    /// Returns a reference to the keyboard.
    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    /// Returns a mutable reference to the keyboard.
    pub fn keyboard_mut(&mut self) -> &mut Keyboard {
        &mut self.keyboard
    }

    /// Returns a reference to the timer.
    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    /// Returns a mutable reference to the timer.
    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }

    /// Returns the total number of cycles executed.
    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    /// Loads a program into memory at the specified address.
    ///
    /// This bypasses the MMU and loads directly into physical memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_hardware::machine::VirtualMachine;
    ///
    /// let mut vm = VirtualMachine::new(1024 * 1024);
    /// let program = vec![0x01, 0x02, 0x03, 0x04];
    /// vm.load_program(0x1000, &program).unwrap();
    /// ```
    pub fn load_program(&mut self, address: Address, program: &[u8]) -> Result<()> {
        self.memory.load(address, program)
    }

    /// Executes one CPU instruction.
    ///
    /// This performs one fetch-decode-execute cycle and updates devices.
    ///
    /// # Returns
    ///
    /// Ok(true) if execution should continue, Ok(false) if halted.
    pub fn step(&mut self) -> Result<bool> {
        // Execute one CPU instruction with memory + I/O access
        let should_continue = self.cpu.step(&mut MemoryBus {
            memory: &mut self.memory,
            display: &mut self.display,
            keyboard: &mut self.keyboard,
            timer: &mut self.timer,
        })?;

        // Update devices (tick timer)
        self.timer.tick(1);
        self.cycles += 1;

        Ok(should_continue)
    }

    /// Runs the virtual machine until it halts or an error occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// use vos_hardware::machine::VirtualMachine;
    /// use vos_cpu::instruction::{Instruction, Opcode};
    ///
    /// let mut vm = VirtualMachine::new(1024 * 1024);
    ///
    /// // Create and load a HALT instruction
    /// let halt = Instruction::IType {
    ///     opcode: Opcode::HALT,
    ///     rt: 0,
    ///     rs: 0,
    ///     immediate: 0,
    /// };
    /// vm.load_program(0, &halt.encode().to_le_bytes()).unwrap();
    /// vm.cpu_mut().set_pc(0);
    ///
    /// vm.run().unwrap();
    /// ```
    pub fn run(&mut self) -> Result<()> {
        while self.step()? {
            // Continue execution
        }
        Ok(())
    }

    /// Runs for a maximum number of cycles.
    ///
    /// Useful for time-slicing or preventing infinite loops.
    ///
    /// # Returns
    ///
    /// Ok(true) if still running, Ok(false) if halted.
    pub fn run_cycles(&mut self, max_cycles: u64) -> Result<bool> {
        for _ in 0..max_cycles {
            if !self.step()? {
                return Ok(false); // Halted
            }
        }
        Ok(true) // Still running
    }

    /// Resets the virtual machine to initial state.
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.memory.clear();
        self.display.reset();
        self.keyboard.reset();
        self.timer.reset();
        self.cycles = 0;
    }

    /// Gets the display output as a string.
    pub fn display_output(&self) -> String {
        self.display.display_to_string()
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new(16 * 1024 * 1024) // 16MB default
    }
}

impl Executable for VirtualMachine {
    fn step(&mut self) -> Result<bool> {
        VirtualMachine::step(self)
    }

    fn run(&mut self) -> Result<()> {
        VirtualMachine::run(self)
    }

    fn reset(&mut self) {
        VirtualMachine::reset(self)
    }
}

impl Inspectable for VirtualMachine {
    fn inspect(&self) -> String {
        let mut output = String::new();

        output.push_str("=== Virtual Machine State ===\n\n");
        output.push_str(&format!("Cycles: {}\n\n", self.cycles));

        output.push_str(&self.cpu.inspect());
        output.push('\n');

        output.push_str(&format!("Memory: {} bytes\n", self.memory.size()));
        output.push_str(&format!(
            "Display: {}x{}\n",
            vos_io::DISPLAY_WIDTH,
            vos_io::DISPLAY_HEIGHT
        ));
        output.push_str(&format!(
            "Keyboard: {} keys buffered\n",
            if self.keyboard.has_data() {
                "data available"
            } else {
                "empty"
            }
        ));
        output.push_str(&format!(
            "Timer: {} (enabled: {})\n",
            self.timer.counter(),
            self.timer.is_enabled()
        ));

        output
    }

    fn state(&self) -> Vec<(String, String)> {
        vec![
            ("Cycles".to_string(), self.cycles.to_string()),
            ("Memory Size".to_string(), self.memory.size().to_string()),
            ("CPU Halted".to_string(), self.cpu.is_halted().to_string()),
            (
                "Timer Enabled".to_string(),
                self.timer.is_enabled().to_string(),
            ),
        ]
    }
}

/// Memory bus that routes accesses to Memory or I/O devices.
///
/// Implements the CPU's Memory trait by routing addresses to the appropriate
/// device based on the memory map.
struct MemoryBus<'a> {
    memory: &'a mut Memory,
    display: &'a mut Display,
    keyboard: &'a mut Keyboard,
    timer: &'a mut Timer,
}

impl<'a> MemoryBus<'a> {
    /// Routes an address to the appropriate device or memory.
    ///
    /// Returns None for memory, or Some(device) for I/O devices.
    fn route_device(&mut self, address: Address) -> Option<&mut dyn Device> {
        // Check if address is in I/O region
        if address >= vos_io::DISPLAY_BASE
            && address < vos_io::DISPLAY_BASE + vos_io::display::DISPLAY_DEVICE_SIZE as u32
        {
            Some(self.display)
        } else if address >= vos_io::KEYBOARD_BASE
            && address < vos_io::KEYBOARD_BASE + vos_io::keyboard::KEYBOARD_SIZE as u32
        {
            Some(self.keyboard)
        } else if address >= vos_io::TIMER_BASE
            && address < vos_io::TIMER_BASE + vos_io::timer::TIMER_SIZE as u32
        {
            Some(self.timer)
        } else {
            None // Regular memory
        }
    }
}

impl<'a> vos_cpu::cpu::Memory for MemoryBus<'a> {
    fn read_word(&mut self, address: Address) -> Result<Word> {
        if let Some(device) = self.route_device(address) {
            // I/O device access
            let base = device.base_address();
            let offset = address - base;
            device.read_word(offset)
        } else {
            // Regular memory access
            self.memory.read_word(address)
        }
    }

    fn write_word(&mut self, address: Address, value: Word) -> Result<()> {
        if let Some(device) = self.route_device(address) {
            // I/O device access
            let base = device.base_address();
            let offset = address - base;
            device.write_word(offset, value)
        } else {
            // Regular memory access
            self.memory.write_word(address, value)
        }
    }

    fn read_byte(&mut self, address: Address) -> Result<Byte> {
        if let Some(device) = self.route_device(address) {
            // I/O device access
            let base = device.base_address();
            let offset = address - base;
            device.read_byte(offset)
        } else {
            // Regular memory access
            self.memory.read_byte(address)
        }
    }

    fn write_byte(&mut self, address: Address, value: Byte) -> Result<()> {
        if let Some(device) = self.route_device(address) {
            // I/O device access
            let base = device.base_address();
            let offset = address - base;
            device.write_byte(offset, value)
        } else {
            // Regular memory access
            self.memory.write_byte(address, value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vos_cpu::instruction::{Instruction, Opcode};

    #[test]
    fn test_vm_creation() {
        let vm = VirtualMachine::new(1024 * 1024);
        assert_eq!(vm.cycles(), 0);
        assert!(!vm.cpu().is_halted());
    }

    #[test]
    fn test_load_program() {
        let mut vm = VirtualMachine::new(1024);
        let program = vec![0x01, 0x02, 0x03, 0x04];

        vm.load_program(0x100, &program).unwrap();

        // Verify program was loaded
        assert_eq!(vm.memory_mut().read_byte(0x100).unwrap(), 0x01);
        assert_eq!(vm.memory_mut().read_byte(0x103).unwrap(), 0x04);
    }

    #[test]
    fn test_execute_halt() {
        let mut vm = VirtualMachine::new(1024);

        // Create HALT instruction
        let halt = Instruction::IType {
            opcode: Opcode::HALT,
            rt: 0,
            rs: 0,
            immediate: 0,
        };

        vm.load_program(0, &[
            (halt.encode() & 0xFF) as u8,
            ((halt.encode() >> 8) & 0xFF) as u8,
            ((halt.encode() >> 16) & 0xFF) as u8,
            ((halt.encode() >> 24) & 0xFF) as u8,
        ])
        .unwrap();

        vm.cpu_mut().set_pc(0);

        // Should halt after one step
        let should_continue = vm.step().unwrap();
        assert!(!should_continue);
        assert!(vm.cpu().is_halted());
    }

    #[test]
    fn test_memory_access() {
        let mut vm = VirtualMachine::new(1024);

        // Write to memory
        vm.memory_mut().write_word(0x100, 0x12345678).unwrap();

        // Read back
        assert_eq!(vm.memory_mut().read_word(0x100).unwrap(), 0x12345678);
    }

    #[test]
    fn test_display_access() {
        let mut vm = VirtualMachine::new(1024);

        // Write to display through display reference
        vm.display_mut().put_char(b'H');
        vm.display_mut().put_char(b'i');

        let output = vm.display_output();
        assert!(output.starts_with("Hi"));
    }

    #[test]
    fn test_keyboard_input() {
        let mut vm = VirtualMachine::new(1024);

        // Simulate key press
        vm.keyboard_mut().push_key(b'A');

        assert!(vm.keyboard().has_data());
    }

    #[test]
    fn test_timer() {
        let mut vm = VirtualMachine::new(1024);

        // Configure timer through device
        vm.timer_mut().write_word(4, 100).unwrap(); // Set reload
        vm.timer_mut().write_byte(8, 0x01).unwrap(); // Enable

        assert!(vm.timer().is_enabled());
        assert_eq!(vm.timer().counter(), 100);
    }

    #[test]
    fn test_reset() {
        let mut vm = VirtualMachine::new(1024);

        // Modify state
        vm.cpu_mut().set_pc(0x1000);
        vm.memory_mut().write_word(0x100, 0x12345678).unwrap();
        vm.display_mut().put_char(b'X');

        // Reset
        vm.reset();

        // Check everything is cleared
        assert_eq!(vm.cpu().pc(), 0);
        assert_eq!(vm.cycles(), 0);
    }

    #[test]
    fn test_run_cycles() {
        let mut vm = VirtualMachine::new(1024);

        // Load NOP instructions
        let nop = Instruction::IType {
            opcode: Opcode::NOP,
            rt: 0,
            rs: 0,
            immediate: 0,
        };

        for i in 0..10 {
            vm.load_program(i * 4, &[
                (nop.encode() & 0xFF) as u8,
                ((nop.encode() >> 8) & 0xFF) as u8,
                ((nop.encode() >> 16) & 0xFF) as u8,
                ((nop.encode() >> 24) & 0xFF) as u8,
            ])
            .unwrap();
        }

        vm.cpu_mut().set_pc(0);

        // Run 5 cycles
        let running = vm.run_cycles(5).unwrap();
        assert!(running);
        assert_eq!(vm.cpu().instruction_count(), 5);
    }
}
