//! CPU implementation with fetch-decode-execute cycle.

use vos_core::{Address, CpuError, Executable, Inspectable, Result, Word};

use crate::alu::Alu;
use crate::instruction::{Funct, Instruction, Opcode};
use crate::registers::Registers;

/// Callback trait for memory access.
///
/// The CPU uses this trait to read from and write to memory.
/// This allows the CPU to be independent of the memory implementation.
pub trait Memory {
    /// Reads a word from memory at the given address.
    fn read_word(&mut self, address: Address) -> Result<Word>;

    /// Writes a word to memory at the given address.
    fn write_word(&mut self, address: Address, value: Word) -> Result<()>;

    /// Reads a byte from memory at the given address.
    fn read_byte(&mut self, address: Address) -> Result<u8>;

    /// Writes a byte to memory at the given address.
    fn write_byte(&mut self, address: Address, value: u8) -> Result<()>;
}

/// The VOS CPU.
///
/// A simple 32-bit RISC processor with 16 registers and ~30 instructions.
///
/// # Architecture
///
/// - 16 general-purpose registers (R0-R15)
/// - 32-bit program counter (PC)
/// - Status flags (zero, negative, carry, overflow)
/// - Simple instruction set (R-type, I-type, J-type)
///
/// # Examples
///
/// ```ignore
/// use vos_cpu::cpu::Cpu;
///
/// let mut cpu = Cpu::new();
/// cpu.set_pc(0x1000);
///
/// // Execute one instruction
/// cpu.step(&mut memory)?;
/// ```
#[derive(Debug)]
pub struct Cpu {
    /// Register file
    pub registers: Registers,

    /// Arithmetic Logic Unit
    alu: Alu,

    /// Is the CPU halted?
    halted: bool,

    /// Total number of instructions executed
    instruction_count: u64,
}

impl Cpu {
    /// Creates a new CPU with all registers initialized to zero.
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            alu: Alu::new(),
            halted: false,
            instruction_count: 0,
        }
    }

    /// Returns the current program counter value.
    pub fn pc(&self) -> Address {
        self.registers.pc
    }

    /// Sets the program counter to a new value.
    pub fn set_pc(&mut self, address: Address) {
        self.registers.pc = address;
    }

    /// Returns true if the CPU is halted.
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Returns the total number of instructions executed.
    pub fn instruction_count(&self) -> u64 {
        self.instruction_count
    }

    /// Executes one instruction (fetch-decode-execute cycle).
    ///
    /// # Steps
    ///
    /// 1. Fetch: Read instruction from memory at PC
    /// 2. Decode: Parse instruction into operation and operands
    /// 3. Execute: Perform the operation
    /// 4. Update: Update PC (usually PC + 4)
    ///
    /// # Returns
    ///
    /// Ok(true) if execution should continue, Ok(false) if halted.
    pub fn step<M: Memory>(&mut self, memory: &mut M) -> Result<bool> {
        if self.halted {
            return Ok(false);
        }

        // Fetch
        let instruction_word = memory.read_word(self.registers.pc)?;
        self.registers.ir = instruction_word;

        // Decode
        let instruction = Instruction::decode(instruction_word)?;

        // Execute
        self.execute(instruction, memory)?;

        // Count instruction
        self.instruction_count += 1;

        Ok(!self.halted)
    }

    /// Executes a decoded instruction.
    fn execute<M: Memory>(&mut self, instruction: Instruction, memory: &mut M) -> Result<()> {
        match instruction {
            Instruction::RType { funct, rd, rs, rt, shamt } => {
                self.execute_rtype(funct, rd, rs, rt, shamt)?;
            }

            Instruction::IType { opcode, rt, rs, immediate } => {
                self.execute_itype(opcode, rt, rs, immediate, memory)?;
            }

            Instruction::JType { opcode, address } => {
                self.execute_jtype(opcode, address)?;
            }
        }

        Ok(())
    }

    /// Executes an R-Type instruction.
    fn execute_rtype(&mut self, funct: Funct, rd: u8, rs: u8, rt: u8, shamt: u8) -> Result<()> {
        let rs_val = self.registers.read(rs);
        let rt_val = self.registers.read(rt);

        let result = match funct {
            Funct::ADD => self.alu.add(rs_val, rt_val)?,
            Funct::SUB => self.alu.sub(rs_val, rt_val)?,
            Funct::MUL => self.alu.mul(rs_val, rt_val)?,
            Funct::DIV => self.alu.div(rs_val, rt_val)?,
            Funct::AND => self.alu.and(rs_val, rt_val)?,
            Funct::OR => self.alu.or(rs_val, rt_val)?,
            Funct::XOR => self.alu.xor(rs_val, rt_val)?,
            Funct::NOT => self.alu.not(rs_val)?,
            Funct::SLL => self.alu.sll(rt_val, shamt)?,
            Funct::SRL => self.alu.srl(rt_val, shamt)?,
            Funct::SRA => self.alu.sra(rt_val, shamt)?,
            Funct::SLT => self.alu.slt(rs_val, rt_val)?,
            Funct::SGT => self.alu.sgt(rs_val, rt_val)?,

            Funct::MOV => {
                // MOV is just setting the value
                self.registers.write(rd, rs_val);
                self.registers.pc += 4;
                return Ok(());
            }

            Funct::JR => {
                // Jump Register: PC = rs
                self.registers.pc = rs_val;
                return Ok(());
            }
        };

        // Write result to destination register
        self.registers.write(rd, result.value);

        // Update flags
        self.registers.flags = result.flags;

        // Advance PC
        self.registers.pc += 4;

        Ok(())
    }

    /// Executes an I-Type instruction.
    fn execute_itype<M: Memory>(
        &mut self,
        opcode: Opcode,
        rt: u8,
        rs: u8,
        immediate: i32,
        memory: &mut M,
    ) -> Result<()> {
        let rs_val = self.registers.read(rs);
        let rt_val = self.registers.read(rt);

        match opcode {
            // Arithmetic immediate
            Opcode::ADDI => {
                let result = self.alu.add(rs_val, immediate as Word)?;
                self.registers.write(rt, result.value);
                self.registers.flags = result.flags;
            }

            Opcode::SUBI => {
                let result = self.alu.sub(rs_val, immediate as Word)?;
                self.registers.write(rt, result.value);
                self.registers.flags = result.flags;
            }

            Opcode::ANDI => {
                let result = self.alu.and(rs_val, immediate as Word)?;
                self.registers.write(rt, result.value);
                self.registers.flags = result.flags;
            }

            Opcode::ORI => {
                let result = self.alu.or(rs_val, immediate as Word)?;
                self.registers.write(rt, result.value);
                self.registers.flags = result.flags;
            }

            Opcode::XORI => {
                let result = self.alu.xor(rs_val, immediate as Word)?;
                self.registers.write(rt, result.value);
                self.registers.flags = result.flags;
            }

            Opcode::LUI => {
                // Load Upper Immediate: rt = immediate << 16
                let value = ((immediate as Word) & 0xFFFF) << 16;
                self.registers.write(rt, value);
            }

            // Memory operations
            Opcode::LW => {
                // Load Word: rt = mem[rs + offset]
                let address = (rs_val as i32 + immediate) as Address;
                let value = memory.read_word(address)?;
                self.registers.write(rt, value);
            }

            Opcode::SW => {
                // Store Word: mem[rs + offset] = rt
                let address = (rs_val as i32 + immediate) as Address;
                memory.write_word(address, rt_val)?;
            }

            Opcode::LB => {
                // Load Byte: rt = mem[rs + offset] (zero-extended)
                let address = (rs_val as i32 + immediate) as Address;
                let byte = memory.read_byte(address)?;
                self.registers.write(rt, byte as Word);
            }

            Opcode::SB => {
                // Store Byte: mem[rs + offset] = rt[7:0]
                let address = (rs_val as i32 + immediate) as Address;
                memory.write_byte(address, (rt_val & 0xFF) as u8)?;
            }

            // Branch instructions
            Opcode::BEQ => {
                // Branch if Equal
                if rs_val == rt_val {
                    self.registers.pc = (self.registers.pc as i32 + (immediate * 4)) as Address;
                    return Ok(());
                }
            }

            Opcode::BNE => {
                // Branch if Not Equal
                if rs_val != rt_val {
                    self.registers.pc = (self.registers.pc as i32 + (immediate * 4)) as Address;
                    return Ok(());
                }
            }

            Opcode::BLT => {
                // Branch if Less Than (signed)
                if (rs_val as i32) < (rt_val as i32) {
                    self.registers.pc = (self.registers.pc as i32 + (immediate * 4)) as Address;
                    return Ok(());
                }
            }

            Opcode::BGT => {
                // Branch if Greater Than (signed)
                if (rs_val as i32) > (rt_val as i32) {
                    self.registers.pc = (self.registers.pc as i32 + (immediate * 4)) as Address;
                    return Ok(());
                }
            }

            Opcode::BLE => {
                // Branch if Less or Equal (signed)
                if (rs_val as i32) <= (rt_val as i32) {
                    self.registers.pc = (self.registers.pc as i32 + (immediate * 4)) as Address;
                    return Ok(());
                }
            }

            Opcode::BGE => {
                // Branch if Greater or Equal (signed)
                if (rs_val as i32) >= (rt_val as i32) {
                    self.registers.pc = (self.registers.pc as i32 + (immediate * 4)) as Address;
                    return Ok(());
                }
            }

            // System instructions
            Opcode::SYSCALL => {
                // System call - to be handled by kernel
                // For now, just advance PC
            }

            Opcode::BREAK => {
                // Breakpoint - for debugging
                // For now, just advance PC
            }

            Opcode::HALT => {
                // Halt execution
                self.halted = true;
                return Ok(());
            }

            Opcode::NOP => {
                // No operation
            }

            _ => {
                return Err(CpuError::InvalidInstruction(self.registers.ir).into());
            }
        }

        // Advance PC (unless branch was taken or halt)
        self.registers.pc += 4;

        Ok(())
    }

    /// Executes a J-Type instruction.
    fn execute_jtype(&mut self, opcode: Opcode, address: u32) -> Result<()> {
        match opcode {
            Opcode::J => {
                // Jump: PC = address
                self.registers.pc = address;
            }

            Opcode::JAL => {
                // Jump and Link: R14 = PC + 4, PC = address
                self.registers.write(14, self.registers.pc + 4);
                self.registers.pc = address;
            }

            _ => {
                return Err(CpuError::InvalidInstruction(self.registers.ir).into());
            }
        }

        Ok(())
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Executable for Cpu {
    fn step(&mut self) -> Result<bool> {
        // For the Executable trait, we need a memory reference
        // This will be properly implemented when we integrate with the VM
        Err(CpuError::InvalidState("CPU requires memory reference".to_string()).into())
    }

    fn reset(&mut self) {
        self.registers.reset();
        self.halted = false;
        self.instruction_count = 0;
    }
}

impl Inspectable for Cpu {
    fn inspect(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("CPU State (Instructions: {})\n", self.instruction_count));
        output.push_str(&format!("Halted: {}\n", self.halted));
        output.push_str("\n");
        output.push_str(&self.registers.dump());
        output
    }

    fn state(&self) -> Vec<(String, String)> {
        let mut state = vec![];

        state.push(("PC".to_string(), format!("0x{:08X}", self.registers.pc)));
        state.push(("Halted".to_string(), self.halted.to_string()));
        state.push((
            "Instructions".to_string(),
            self.instruction_count.to_string(),
        ));

        for i in 0..16 {
            state.push((
                format!("R{}", i),
                format!("0x{:08X}", self.registers.read(i)),
            ));
        }

        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock memory for testing
    struct MockMemory {
        data: Vec<Word>,
    }

    impl MockMemory {
        fn new(size: usize) -> Self {
            Self {
                data: vec![0; size / 4],
            }
        }

        fn load_program(&mut self, start_addr: Address, program: &[Word]) {
            let start_idx = (start_addr / 4) as usize;
            for (i, &word) in program.iter().enumerate() {
                self.data[start_idx + i] = word;
            }
        }
    }

    impl Memory for MockMemory {
        fn read_word(&mut self, address: Address) -> Result<Word> {
            let idx = (address / 4) as usize;
            Ok(self.data.get(idx).copied().unwrap_or(0))
        }

        fn write_word(&mut self, address: Address, value: Word) -> Result<()> {
            let idx = (address / 4) as usize;
            if idx < self.data.len() {
                self.data[idx] = value;
            }
            Ok(())
        }

        fn read_byte(&mut self, address: Address) -> Result<u8> {
            let word = self.read_word(address & !3)?;
            let byte_offset = (address & 3) as usize;
            Ok(((word >> (byte_offset * 8)) & 0xFF) as u8)
        }

        fn write_byte(&mut self, address: Address, value: u8) -> Result<()> {
            let word_addr = address & !3;
            let mut word = self.read_word(word_addr)?;
            let byte_offset = (address & 3) as usize;

            let mask = !(0xFF << (byte_offset * 8));
            word = (word & mask) | ((value as Word) << (byte_offset * 8));

            self.write_word(word_addr, word)
        }
    }

    #[test]
    fn test_cpu_creation() {
        let cpu = Cpu::new();
        assert_eq!(cpu.pc(), 0);
        assert!(!cpu.is_halted());
        assert_eq!(cpu.instruction_count(), 0);
    }

    #[test]
    fn test_add_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = MockMemory::new(1024);

        // ADDI R1, R0, 10   (R1 = 10)
        let instr = Instruction::IType {
            opcode: Opcode::ADDI,
            rt: 1,
            rs: 0,
            immediate: 10,
        };
        memory.load_program(0, &[instr.encode()]);

        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.registers.read(1), 10);
        assert_eq!(cpu.pc(), 4);
    }

    #[test]
    fn test_add_rtype() {
        let mut cpu = Cpu::new();
        let mut memory = MockMemory::new(1024);

        // Setup: R2 = 10, R3 = 20
        cpu.registers.write(2, 10);
        cpu.registers.write(3, 20);

        // ADD R1, R2, R3  (R1 = R2 + R3)
        let instr = Instruction::RType {
            funct: Funct::ADD,
            rd: 1,
            rs: 2,
            rt: 3,
            shamt: 0,
        };
        memory.load_program(0, &[instr.encode()]);

        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.registers.read(1), 30);
    }

    #[test]
    fn test_load_store() {
        let mut cpu = Cpu::new();
        let mut memory = MockMemory::new(1024);

        // Setup: R2 = address 0x100
        cpu.registers.write(2, 0x100);

        // SW R1, 0(R2)  - Store R1 (0) to memory[0x100]
        let store = Instruction::IType {
            opcode: Opcode::SW,
            rt: 1,
            rs: 2,
            immediate: 0,
        };

        // Write value to R1 first
        cpu.registers.write(1, 42);

        memory.load_program(0, &[store.encode()]);
        cpu.step(&mut memory).unwrap();

        // LW R3, 0(R2)  - Load from memory[0x100] to R3
        cpu.set_pc(4);
        let load = Instruction::IType {
            opcode: Opcode::LW,
            rt: 3,
            rs: 2,
            immediate: 0,
        };
        memory.load_program(4, &[load.encode()]);
        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.registers.read(3), 42);
    }

    #[test]
    fn test_branch() {
        let mut cpu = Cpu::new();
        let mut memory = MockMemory::new(1024);

        // Setup: R1 = 10, R2 = 10
        cpu.registers.write(1, 10);
        cpu.registers.write(2, 10);

        // BEQ R1, R2, 4  (branch to PC + 16)
        let instr = Instruction::IType {
            opcode: Opcode::BEQ,
            rt: 2,
            rs: 1,
            immediate: 4, // Offset in words
        };
        memory.load_program(0, &[instr.encode()]);

        cpu.step(&mut memory).unwrap();

        // Branch taken: PC = 0 + (4 * 4) = 16
        assert_eq!(cpu.pc(), 16);
    }

    #[test]
    fn test_halt() {
        let mut cpu = Cpu::new();
        let mut memory = MockMemory::new(1024);

        // HALT instruction
        let instr = Instruction::IType {
            opcode: Opcode::HALT,
            rt: 0,
            rs: 0,
            immediate: 0,
        };
        memory.load_program(0, &[instr.encode()]);

        let should_continue = cpu.step(&mut memory).unwrap();

        assert!(!should_continue);
        assert!(cpu.is_halted());
    }

    #[test]
    fn test_jump() {
        let mut cpu = Cpu::new();
        let mut memory = MockMemory::new(1024);

        // J 0x1000
        let instr = Instruction::JType {
            opcode: Opcode::J,
            address: 0x1000,
        };
        memory.load_program(0, &[instr.encode()]);

        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.pc(), 0x1000);
    }
}
