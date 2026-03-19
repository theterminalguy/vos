# Chapter 2: CPU Basics

## Learning Objectives

By the end of this chapter, you will:
- Understand the fetch-decode-execute cycle
- Learn about registers, the ALU, and instruction formats
- Write and execute simple assembly programs
- Understand how instructions are encoded as binary

## Introduction

The CPU (Central Processing Unit) is the brain of the computer. It fetches instructions from memory, decodes them to understand what operation to perform, and executes them. In this chapter, we'll explore VOS's 32-bit RISC CPU architecture and learn how it processes instructions.

## CPU Architecture Overview

The VOS CPU is a 32-bit RISC (Reduced Instruction Set Computer) processor with:

- **16 General-Purpose Registers** (R0-R15)
  - R0: Always contains zero (hardwired)
  - R1-R14: General purpose
  - R15: Stack pointer (by convention)

- **Special Registers**
  - PC (Program Counter): Points to the next instruction to execute
  - IR (Instruction Register): Holds the current instruction
  - FLAGS: Status flags (Zero, Negative, Carry, Overflow)

- **32-bit Architecture**
  - 32-bit words (4 bytes)
  - 32-bit address space (4GB addressable)
  - Little-endian byte order

## The Fetch-Decode-Execute Cycle

Every instruction execution follows three steps:

### 1. Fetch
```rust
// Read instruction from memory at PC
let instruction_word = memory.read_word(self.registers.pc)?;
self.registers.ir = instruction_word;
```

The CPU reads 4 bytes (32 bits) from memory at the address stored in the Program Counter (PC).

### 2. Decode
```rust
// Decode the instruction
let instruction = Instruction::decode(instruction_word)?;
```

The CPU interprets the bit pattern to determine:
- What operation to perform (ADD, SUB, LOAD, etc.)
- Which registers are involved
- What immediate values (if any) to use

### 3. Execute
```rust
// Execute the instruction
match instruction {
    Instruction::RType { funct, rd, rs, rt, .. } => {
        // Execute R-type instruction (register operations)
        let result = self.alu.execute(funct, rs_value, rt_value)?;
        self.registers.write(rd, result.value)?;
    }
    Instruction::IType { opcode, rt, rs, immediate } => {
        // Execute I-type instruction (immediate operations)
        // ...
    }
    // ... more instruction types
}
```

The CPU performs the operation and updates registers, memory, or the PC as needed.

## Instruction Formats

VOS uses three instruction formats:

### R-Type (Register): 32 bits
```
[opcode: 6][rs: 4][rt: 4][rd: 4][shamt: 5][funct: 9]
```

Used for operations between three registers:
- `rd = rs OP rt`
- Example: `ADD R3, R1, R2` → R3 = R1 + R2

**Example encoding:**
```rust
Instruction::RType {
    funct: Funct::ADD,
    rd: 3,    // destination
    rs: 1,    // source 1
    rt: 2,    // source 2
    shamt: 0, // shift amount (unused for ADD)
}
```

### I-Type (Immediate): 32 bits
```
[opcode: 6][rs: 4][rt: 4][immediate: 18]
```

Used for operations with a constant value:
- `rt = rs OP immediate`
- Example: `ADDI R1, R0, 42` → R1 = R0 + 42 = 42

**Example encoding:**
```rust
Instruction::IType {
    opcode: Opcode::ADDI,
    rt: 1,          // destination
    rs: 0,          // source
    immediate: 42,  // constant value
}
```

### J-Type (Jump): 32 bits
```
[opcode: 6][address: 26]
```

Used for jumps to absolute addresses:
- Example: `J 0x1000` → PC = 0x1000

**Example encoding:**
```rust
Instruction::JType {
    opcode: Opcode::J,
    address: 0x1000,
}
```

## Register File

The register file stores 16 general-purpose registers:

```rust
pub struct Registers {
    gpr: [Word; 16],  // General-purpose registers
    pub pc: Address,   // Program counter
    pub ir: Word,      // Instruction register
    pub flags: Flags,  // Status flags
}
```

### Special Register: R0

R0 is **always zero**. Any writes to R0 are ignored:

```rust
pub fn write(&mut self, index: RegisterIndex, value: Word) -> Result<()> {
    if index == 0 {
        return Ok(()); // R0 is always zero
    }
    self.gpr[index as usize] = value;
    Ok(())
}
```

This makes R0 useful for operations like:
- `ADDI R1, R0, 42` → Load 42 into R1
- `ADD R2, R1, R0` → Copy R1 to R2

## The ALU (Arithmetic Logic Unit)

The ALU performs all arithmetic and logical operations:

```rust
pub struct Alu;

impl Alu {
    pub fn add(&self, a: Word, b: Word) -> Result<AluResult> {
        let result = a.wrapping_add(b);
        let mut flags = Flags::new();

        // Set Zero flag if result is 0
        flags.zero = result == 0;

        // Set Negative flag if result is negative (bit 31 set)
        flags.negative = (result as i32) < 0;

        // Set Carry flag if addition overflowed
        flags.carry = result < a;

        // Set Overflow flag for signed overflow
        let a_sign = (a as i32) < 0;
        let b_sign = (b as i32) < 0;
        let r_sign = (result as i32) < 0;
        flags.overflow = (a_sign == b_sign) && (a_sign != r_sign);

        Ok(AluResult { value: result, flags })
    }
}
```

### CPU Flags

After most operations, the ALU updates status flags:

- **Zero (Z)**: Set if result is 0
- **Negative (N)**: Set if result is negative (bit 31 = 1)
- **Carry (C)**: Set if unsigned overflow occurred
- **Overflow (V)**: Set if signed overflow occurred

These flags are used by branch instructions like:
- `BEQ` - Branch if Equal (Z flag set)
- `BNE` - Branch if Not Equal (Z flag clear)
- `BLT` - Branch if Less Than (N flag set)

## Instruction Set Categories

### 1. Arithmetic Operations
```
ADD  R1, R2, R3    ; R1 = R2 + R3
SUB  R1, R2, R3    ; R1 = R2 - R3
ADDI R1, R2, 10    ; R1 = R2 + 10
SUBI R1, R2, 5     ; R1 = R2 - 5
MUL  R1, R2, R3    ; R1 = R2 * R3
DIV  R1, R2, R3    ; R1 = R2 / R3
```

### 2. Logical Operations
```
AND  R1, R2, R3    ; R1 = R2 & R3
OR   R1, R2, R3    ; R1 = R2 | R3
XOR  R1, R2, R3    ; R1 = R2 ^ R3
NOT  R1, R2, R3    ; R1 = ~R2 (R3 unused)
ANDI R1, R2, 0xFF  ; R1 = R2 & 0xFF
```

### 3. Shift Operations
```
SLL  R1, R2, R3    ; R1 = R2 << R3 (shift left logical)
SRL  R1, R2, R3    ; R1 = R2 >> R3 (shift right logical)
SRA  R1, R2, R3    ; R1 = R2 >> R3 (shift right arithmetic)
```

### 4. Comparison Operations
```
SLT  R1, R2, R3    ; R1 = (R2 < R3) ? 1 : 0
SGT  R1, R2, R3    ; R1 = (R2 > R3) ? 1 : 0
```

### 5. Memory Operations
```
LW   R1, R2, 0     ; R1 = Memory[R2 + 0] (load word)
SW   R1, R2, 0     ; Memory[R2 + 0] = R1 (store word)
LB   R1, R2, 0     ; R1 = Memory[R2 + 0] (load byte)
SB   R1, R2, 0     ; Memory[R2 + 0] = R1 (store byte)
LUI  R1, 0x8000    ; R1 = 0x80000000 (load upper immediate)
```

### 6. Control Flow
```
J    label         ; Jump to label
JAL  label         ; Jump and link (save return address)
BEQ  R1, R2, label ; Branch if R1 == R2
BNE  R1, R2, label ; Branch if R1 != R2
BLT  R1, R2, label ; Branch if R1 < R2
BGT  R1, R2, label ; Branch if R1 > R2
```

### 7. System Operations
```
HALT               ; Stop execution
NOP                ; No operation
SYSCALL            ; System call
BREAK              ; Breakpoint
```

## Example: Adding Two Numbers

Let's trace through a simple program that adds 10 and 20:

```assembly
ADDI R1, R0, 10    ; R1 = 10
ADDI R2, R0, 20    ; R2 = 20
ADD  R3, R1, R2    ; R3 = R1 + R2 = 30
HALT               ; Stop
```

### Instruction 1: `ADDI R1, R0, 10`

**Fetch:**
- PC = 0x0000
- Read instruction at 0x0000

**Decode:**
- Opcode: ADDI (Add Immediate)
- rt = 1 (destination)
- rs = 0 (source = R0 = 0)
- immediate = 10

**Execute:**
- R1 = R0 + 10 = 0 + 10 = 10
- PC = PC + 4 = 0x0004

### Instruction 2: `ADDI R2, R0, 20`

**Fetch:**
- PC = 0x0004
- Read instruction at 0x0004

**Decode:**
- Opcode: ADDI
- rt = 2
- rs = 0
- immediate = 20

**Execute:**
- R2 = R0 + 20 = 0 + 20 = 20
- PC = PC + 4 = 0x0008

### Instruction 3: `ADD R3, R1, R2`

**Fetch:**
- PC = 0x0008
- Read instruction at 0x0008

**Decode:**
- Funct: ADD
- rd = 3 (destination)
- rs = 1 (R1 = 10)
- rt = 2 (R2 = 20)

**Execute:**
- R3 = R1 + R2 = 10 + 20 = 30
- Flags: Z=0, N=0, C=0, V=0
- PC = PC + 4 = 0x000C

### Instruction 4: `HALT`

**Fetch:**
- PC = 0x000C
- Read instruction at 0x000C

**Decode:**
- Opcode: HALT

**Execute:**
- Set halted flag = true
- Execution stops

## Hands-On Exercise: Fibonacci Calculator

Write a program that calculates the 5th Fibonacci number.

**Requirements:**
- Store F(0) = 0 in R1
- Store F(1) = 1 in R2
- Calculate F(2) through F(5)
- Store final result in R6

**Solution:**

```assembly
; Initialize first two Fibonacci numbers
ADDI R1, R0, 0      ; F(0) = 0
ADDI R2, R0, 1      ; F(1) = 1

; Calculate F(2) = F(0) + F(1)
ADD  R3, R1, R2     ; R3 = 0 + 1 = 1

; Update for next iteration
ADD  R1, R0, R2     ; R1 = F(1) = 1
ADD  R2, R0, R3     ; R2 = F(2) = 1

; Calculate F(3) = F(1) + F(2)
ADD  R4, R1, R2     ; R4 = 1 + 1 = 2

; Update for next iteration
ADD  R1, R0, R2     ; R1 = F(2) = 1
ADD  R2, R0, R4     ; R2 = F(3) = 2

; Calculate F(4) = F(2) + F(3)
ADD  R5, R1, R2     ; R5 = 1 + 2 = 3

; Update for next iteration
ADD  R1, R0, R2     ; R1 = F(3) = 2
ADD  R2, R0, R5     ; R2 = F(4) = 3

; Calculate F(5) = F(3) + F(4)
ADD  R6, R1, R2     ; R6 = 2 + 3 = 5

HALT                ; F(5) = 5 is in R6
```

## Code Walkthrough: CPU Implementation

Let's look at the key parts of the CPU implementation:

### CPU Structure

```rust
pub struct Cpu {
    pub registers: Registers,  // Register file
    alu: Alu,                  // Arithmetic logic unit
    halted: bool,              // Halt flag
    instruction_count: u64,    // Performance counter
}
```

### Step Function

```rust
pub fn step<M: Memory>(&mut self, memory: &mut M) -> Result<bool> {
    if self.halted {
        return Ok(false);
    }

    // 1. FETCH: Read instruction from memory
    let instruction_word = memory.read_word(self.registers.pc)?;
    self.registers.ir = instruction_word;

    // 2. DECODE: Parse instruction
    let instruction = Instruction::decode(instruction_word)?;

    // 3. EXECUTE: Run instruction
    self.execute(instruction, memory)?;

    self.instruction_count += 1;
    Ok(!self.halted)
}
```

### Execute Function (Simplified)

```rust
fn execute<M: Memory>(&mut self, instruction: Instruction, memory: &mut M) -> Result<()> {
    match instruction {
        Instruction::RType { funct, rd, rs, rt, .. } => {
            let rs_val = self.registers.read(rs);
            let rt_val = self.registers.read(rt);

            let result = match funct {
                Funct::ADD => self.alu.add(rs_val, rt_val)?,
                Funct::SUB => self.alu.sub(rs_val, rt_val)?,
                // ... other operations
            };

            self.registers.write(rd, result.value)?;
            self.registers.flags = result.flags;
            self.registers.pc += 4;
        }

        Instruction::IType { opcode, rt, rs, immediate } => {
            match opcode {
                Opcode::ADDI => {
                    let rs_val = self.registers.read(rs);
                    let result = self.alu.add(rs_val, immediate as u32)?;
                    self.registers.write(rt, result.value)?;
                    self.registers.flags = result.flags;
                    self.registers.pc += 4;
                }
                Opcode::HALT => {
                    self.halted = true;
                }
                // ... other opcodes
            }
        }

        // ... other instruction types
    }

    Ok(())
}
```

## Challenge Problems

### Challenge 1: Maximum of Three Numbers

Write a program that finds the maximum of three numbers stored in R1, R2, and R3, and stores the result in R10.

**Hint:** Use comparison and conditional operations.

### Challenge 2: Array Sum

Write a program that sums 5 consecutive numbers starting from 1 (1+2+3+4+5) and stores the result in R10.

**Hint:** Use a loop with a counter.

### Challenge 3: Bit Manipulation

Write a program that:
1. Loads 0x12345678 into R1
2. Extracts the second byte (0x56) using shifts and masks
3. Stores the result in R2

**Hint:** Use SRL (shift right) and ANDI (and immediate).

## Summary

In this chapter, you learned:

✅ The fetch-decode-execute cycle is the heart of CPU operation
✅ VOS has three instruction formats: R-type, I-type, and J-type
✅ The register file stores 16 registers, with R0 hardwired to zero
✅ The ALU performs arithmetic and logic operations and updates flags
✅ Instructions are encoded as 32-bit binary values
✅ The CPU implementation follows a clear structure: fetch → decode → execute

## Next Steps

In Chapter 3, we'll explore the **memory system**: how the CPU accesses RAM, how virtual memory works with the MMU, and how paging enables memory protection and isolation.

## Further Reading

- `crates/vos-cpu/src/cpu.rs` - Complete CPU implementation
- `crates/vos-cpu/src/instruction.rs` - Instruction encoding/decoding
- `crates/vos-cpu/src/alu.rs` - ALU operations
- `crates/vos-cpu/src/registers.rs` - Register file implementation

## Testing Your Understanding

1. What happens if you try to write a value to R0?
2. Why is the instruction size 4 bytes?
3. What's the difference between SRL and SRA?
4. How does the CPU know when to stop executing instructions?
5. What flags are set after executing `SUB R1, R1, R1`?

**Answers:**
1. The write is ignored; R0 always reads as 0
2. All instructions are 32 bits (4 bytes) for simplicity and alignment
3. SRL is logical (fills with 0s), SRA is arithmetic (preserves sign bit)
4. When it executes a HALT instruction or encounters an error
5. Zero flag is set (result is 0); other flags depend on implementation
