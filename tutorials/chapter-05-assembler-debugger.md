# Chapter 5: Assembler and Debugger

## Learning Objectives

By the end of this chapter, you will:
- Write programs in VOS assembly language
- Understand how assembly code is converted to machine code
- Use the assembler to compile programs
- Debug programs interactively with the VOS debugger
- Set breakpoints and inspect program state
- Troubleshoot common assembly errors

## Introduction

Writing programs in raw machine code (0s and 1s) is tedious and error-prone. Assembly language provides a human-readable representation of machine instructions. The **assembler** converts assembly code to machine code, and the **debugger** helps you find and fix bugs by letting you step through programs and inspect their state.

## Assembly Language Syntax

VOS assembly uses a simple, readable syntax:

### Basic Syntax Rules

```assembly
; Comments start with semicolon
label:              ; Labels end with colon
    MNEMONIC operands   ; Instruction with operands
```

### Instruction Formats

**R-Type (3 registers):**
```assembly
ADD  R1, R2, R3     ; R1 = R2 + R3
SUB  R4, R5, R6     ; R4 = R5 - R6
MUL  R7, R8, R9     ; R7 = R8 * R9
```

**I-Type (register + immediate):**
```assembly
ADDI R1, R2, 42     ; R1 = R2 + 42
LW   R3, R4, 100    ; R3 = Memory[R4 + 100]
BEQ  R5, R6, loop   ; if R5 == R6 goto loop
```

**J-Type (jump):**
```assembly
J    start          ; Jump to label 'start'
JAL  function       ; Jump and link (function call)
```

**Zero-Operand:**
```assembly
HALT                ; Stop execution
NOP                 ; No operation
```

### Numeric Formats

```assembly
ADDI R1, R0, 42     ; Decimal: 42
ADDI R1, R0, 0x2A   ; Hexadecimal: 0x2A (42)
ADDI R1, R0, -10    ; Negative: -10
```

### Labels

Labels provide symbolic names for addresses:

```assembly
start:
    ADDI R1, R0, 10

loop:
    SUBI R1, R1, 1
    BNE  R1, R0, loop    ; Jump to 'loop' label

end:
    HALT
```

The assembler automatically calculates label addresses.

## The Assembler Pipeline

Assembly code goes through three stages:

```
Source Code → Lexer → Tokens → Parser → AST → Assembler → Machine Code
```

### Stage 1: Lexer (Tokenization)

The lexer breaks source code into tokens:

**Input:**
```assembly
ADDI R1, R0, 42
```

**Output (tokens):**
```
[Mnemonic("ADDI"), Register(1), Comma, Register(0), Comma, Number(42), Newline]
```

**Lexer Implementation:**
```rust
pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.current_char() {
            'R' | 'r' => self.read_register(),
            '0'..='9' | '-' => self.read_number(),
            'A'..='Z' | 'a'..='z' => self.read_identifier(),
            ',' => Token::Comma,
            ';' => { self.skip_comment(); self.next_token() }
            _ => Token::Eof,
        }
    }
}
```

### Stage 2: Parser (AST Generation)

The parser converts tokens into an Abstract Syntax Tree (AST):

**Tokens:**
```
[Mnemonic("ADDI"), Register(1), Register(0), Number(42)]
```

**AST:**
```rust
AsmInstruction::IType {
    mnemonic: "ADDI",
    rt: 1,
    rs: 0,
    immediate: 42,
}
```

**Parser Implementation:**
```rust
impl Parser {
    fn parse_itype(&mut self, mnemonic: String) -> Result<AsmInstruction> {
        let rt = self.expect_register()?;
        self.expect_comma()?;
        let rs = self.expect_register()?;
        self.expect_comma()?;
        let immediate = self.expect_number()?;

        Ok(AsmInstruction::IType {
            mnemonic,
            rt,
            rs,
            immediate,
        })
    }
}
```

### Stage 3: Assembler (Code Generation)

The assembler converts AST to machine code:

**AST:**
```rust
AsmInstruction::IType {
    mnemonic: "ADDI",
    rt: 1,
    rs: 0,
    immediate: 42,
}
```

**Machine Code:**
```
0x0000002A04000001  (as bytes: [0x01, 0x00, 0x00, 0x04, 0x2A, 0x00, 0x00, 0x00])
```

**Assembler Implementation:**
```rust
impl Assembler {
    fn assemble_instruction(&self, instr: &AsmInstruction) -> Result<Instruction> {
        match instr {
            AsmInstruction::IType { mnemonic, rt, rs, immediate } => {
                let opcode = self.mnemonic_to_opcode(mnemonic)?;
                Ok(Instruction::IType {
                    opcode,
                    rt: *rt,
                    rs: *rs,
                    immediate: *immediate,
                })
            }
            // ... other types
        }
    }
}
```

## Using the Assembler

### From Code

```rust
use vos_asm::assemble;

let source = r#"
    ; Calculate 10 + 20
    ADDI R1, R0, 10
    ADDI R2, R0, 20
    ADD  R3, R1, R2
    HALT
"#;

let machine_code = assemble(source)?;
// machine_code is Vec<u8> ready to load into memory
```

### Label Resolution

The assembler resolves labels to addresses:

```assembly
start:
    ADDI R1, R0, 5      ; Address 0x0000

loop:
    SUBI R1, R1, 1      ; Address 0x0004
    BNE  R1, R0, loop   ; Jump to 0x0004
    HALT                ; Address 0x000C
```

When the assembler sees `BNE R1, R0, loop`:
1. Look up 'loop' in label table → address 0x0004
2. Calculate relative offset if needed
3. Encode address in instruction

## Example Programs

### Example 1: Fibonacci

Calculate the 5th Fibonacci number:

```assembly
; Fibonacci: Calculate F(5)
start:
    ; Initialize F(0) = 0, F(1) = 1
    ADDI R1, R0, 0      ; R1 = 0
    ADDI R2, R0, 1      ; R2 = 1

    ; Calculate F(2) = F(0) + F(1)
    ADD  R3, R1, R2     ; R3 = 1

    ; Shift: R1=F(1), R2=F(2)
    ADD  R1, R0, R2
    ADD  R2, R0, R3

    ; Calculate F(3)
    ADD  R4, R1, R2     ; R4 = 2

    ; Shift
    ADD  R1, R0, R2
    ADD  R2, R0, R4

    ; Calculate F(4)
    ADD  R5, R1, R2     ; R5 = 3

    ; Shift
    ADD  R1, R0, R2
    ADD  R2, R0, R5

    ; Calculate F(5)
    ADD  R6, R1, R2     ; R6 = 5

    HALT
; F(5) = 5 is in R6
```

### Example 2: Loop with Counter

Count down from 10 to 0:

```assembly
; Countdown from 10
start:
    ADDI R1, R0, 10     ; R1 = counter = 10

loop:
    SUBI R1, R1, 1      ; Decrement counter
    BNE  R1, R0, loop   ; Loop while counter != 0

done:
    HALT                ; R1 = 0
```

### Example 3: Array Sum

Sum array of 5 numbers:

```assembly
; Sum array: [10, 20, 30, 40, 50]
; Result should be 150

start:
    ; Initialize sum
    ADDI R10, R0, 0     ; R10 = sum = 0

    ; Add each element
    ADDI R1, R0, 10
    ADD  R10, R10, R1

    ADDI R2, R0, 20
    ADD  R10, R10, R2

    ADDI R3, R0, 30
    ADD  R10, R10, R3

    ADDI R4, R0, 40
    ADD  R10, R10, R4

    ADDI R5, R0, 50
    ADD  R10, R10, R5

    HALT
; R10 = 150
```

## The VOS Debugger

The debugger provides interactive program execution and inspection.

### Debugger Features

- **Step Execution**: Run one instruction at a time
- **Breakpoints**: Pause at specific addresses
- **State Inspection**: View registers, memory, flags
- **Disassembly**: See what instruction is executing
- **Continue**: Run until breakpoint or halt

### Debugger Structure

```rust
pub struct Debugger {
    vm: VirtualMachine,             // The VM being debugged
    breakpoints: Vec<Breakpoint>,   // Active breakpoints
    running: bool,                   // Debugger state
}

pub struct Breakpoint {
    number: usize,    // Breakpoint ID
    address: Address, // Where to break
    enabled: bool,    // Is it active?
}
```

### Using the Debugger

```rust
use vos_debugger::Debugger;
use vos_hardware::VirtualMachine;

// Create VM and load program
let mut vm = VirtualMachine::new(1024 * 1024);
vm.load_program(0, &machine_code)?;
vm.cpu_mut().set_pc(0);

// Create debugger
let mut debugger = Debugger::new(vm);

// Set breakpoint at address 0x10
debugger.set_breakpoint(0x10);

// Step through first instruction
debugger.step()?;

// View registers
debugger.info_registers();

// Continue to breakpoint
debugger.continue_exec()?;

// Inspect memory
debugger.info_memory(0x1000, 4)?;

// Disassemble current instruction
debugger.disassemble()?;
```

## Debugger Commands

### step (s)

Execute one instruction:

```
(vos-dbg) step
0x00000004: ADDI R2, R0, 20 (0x00140004)
```

After stepping:
- PC advances to next instruction
- Registers/flags update
- Devices tick

### continue (c)

Run until breakpoint or halt:

```
(vos-dbg) continue
Breakpoint at 0x00000010
```

The debugger runs at full speed until:
- A breakpoint is hit
- Program halts
- An error occurs

### break <addr>

Set a breakpoint:

```
(vos-dbg) break 0x10
Breakpoint 1 at 0x00000010
```

When PC reaches 0x10, execution pauses.

### delete <num>

Remove a breakpoint:

```
(vos-dbg) delete 1
Deleted breakpoint 1 at 0x00000010
```

### list

Show all breakpoints:

```
(vos-dbg) list
Num  Address      Enabled
---  ----------   -------
1    0x00000010   yes
2    0x00000020   yes
```

### info registers

Display all registers and flags:

```
(vos-dbg) info registers

=== Registers ===
R0  = 0x00000000 (0)
R1  = 0x0000000A (10)
R2  = 0x00000014 (20)
R3  = 0x0000001E (30)
...
R15 = 0x00000000 (0)

PC  = 0x00000008

=== Flags ===
Zero     (Z): false
Negative (N): false
Carry    (C): false
Overflow (V): false
```

### info memory <addr> [count]

Display memory contents:

```
(vos-dbg) info memory 0x1000 4

=== Memory at 0x00001000 ===
0x00001000: 0x0A000104  10 1 0 10
0x00001004: 0x14000204  20 2 0 20
0x00001008: 0x00311203   3 18 49 0
0x0000100C: 0x00000080 128 0 0 0
```

Format: `address: word (hex)  bytes (decimal)`

### disassemble

Show current instruction:

```
(vos-dbg) disassemble
0x00000004: ADDI R2, R0, 20 (0x00140004)
```

Shows:
- Address (PC)
- Disassembled instruction
- Raw machine code

### quit (q)

Exit debugger:

```
(vos-dbg) quit
```

## Debugging Workflow

### Example: Debugging a Buggy Program

**Buggy Program:**
```assembly
; Supposed to calculate 10 + 20 = 30
; But has a bug!

start:
    ADDI R1, R0, 10     ; R1 = 10
    ADDI R2, R0, 20     ; R2 = 20
    ADD  R3, R1, R1     ; BUG: Should be ADD R3, R1, R2
    HALT
```

**Debug Session:**

```
(vos-dbg) break 0xC
Breakpoint 1 at 0x0000000C

(vos-dbg) continue
Breakpoint at 0x0000000C

(vos-dbg) info registers
R1  = 0x0000000A (10)
R2  = 0x00000014 (20)
R3  = 0x00000014 (20)  ← Wrong! Should be 30

(vos-dbg) disassemble
0x00000008: ADD R3, R1, R1 (0x...)  ← Found the bug!
```

The bug is clear: `ADD R3, R1, R1` adds R1 to itself (10 + 10 = 20) instead of adding R1 and R2.

**Fixed Program:**
```assembly
start:
    ADDI R1, R0, 10
    ADDI R2, R0, 20
    ADD  R3, R1, R2     ; Fixed: R3 = R1 + R2
    HALT
```

## Hands-On Exercise: Debug the Mystery Program

This program should calculate the factorial of 5 (5! = 120), but it produces the wrong result. Use the debugger to find and fix the bug.

```assembly
; Calculate 5! (factorial of 5)
; Expected result: 120
; Actual result: ???

start:
    ADDI R1, R0, 5      ; N = 5
    ADDI R2, R0, 1      ; Result = 1

loop:
    MUL  R2, R2, R1     ; Result *= N
    SUBI R1, R1, 1      ; N--
    BNE  R1, R0, loop   ; BUG: Should be BGT or check for R1 > 0

    HALT
; R2 should be 120
```

**Debug Steps:**

1. Load program in debugger
2. Set breakpoint at HALT
3. Run to breakpoint
4. Check R2 value
5. If wrong, step through loop
6. Find where calculation goes wrong
7. Fix the bug

<details>
<summary>Click for solution</summary>

The bug: The loop runs one extra time when R1 = 0, multiplying by 0 and zeroing the result.

**Fix:** The condition should stop BEFORE R1 reaches 0:

```assembly
loop:
    MUL  R2, R2, R1
    SUBI R1, R1, 1
    BGT  R1, R0, loop   ; Changed: Branch if Greater Than
    HALT
```

Or check R1 at the start:

```assembly
loop:
    BEQ  R1, R0, done   ; Exit if R1 == 0
    MUL  R2, R2, R1
    SUBI R1, R1, 1
    J    loop

done:
    HALT
```

</details>

## Code Walkthrough: Debugger Implementation

### Step Execution

```rust
pub fn step(&mut self) -> Result<bool> {
    self.vm.step()  // Execute one instruction
}
```

Simple! Just delegates to the VirtualMachine's step() method.

### Continue with Breakpoint Check

```rust
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
```

### Disassembly

```rust
pub fn disassemble(&mut self) -> Result<()> {
    let pc = self.vm.cpu().pc();
    let word = self.vm.memory_mut().read_word(pc)?;

    match Instruction::decode(word) {
        Ok(instr) => {
            println!("0x{:08X}: {} (0x{:08X})",
                     pc, instr.disassemble(), word);
        }
        Err(e) => {
            println!("0x{:08X}: Invalid instruction - {}", pc, e);
        }
    }

    Ok(())
}
```

## Common Assembly Errors

### 1. Wrong Register in Operation

```assembly
ADD R3, R1, R1    ; BUG: Adds R1 to itself
ADD R3, R1, R2    ; FIX: Adds R1 and R2
```

### 2. Forgetting to Initialize

```assembly
; BUG: R1 is used but never initialized
ADD R2, R1, R3    ; R1 contains garbage

; FIX: Initialize first
ADDI R1, R0, 10
ADD  R2, R1, R3
```

### 3. Off-by-One in Loops

```assembly
; BUG: Loops one extra time (multiplies by 0)
loop:
    MUL  R2, R2, R1
    SUBI R1, R1, 1
    BNE  R1, R0, loop

; FIX: Check before multiplying
loop:
    BEQ  R1, R0, done
    MUL  R2, R2, R1
    SUBI R1, R1, 1
    J    loop
done:
```

### 4. Wrong Branch Condition

```assembly
BEQ R1, R2, label  ; Branch if Equal
BNE R1, R2, label  ; Branch if Not Equal
BLT R1, R2, label  ; Branch if Less Than
BGT R1, R2, label  ; Branch if Greater Than
```

Make sure you use the right comparison!

## Challenge Problems

### Challenge 1: Maximum Finder

Write a program that finds the maximum of three numbers (stored in R1, R2, R3) and stores it in R10.

Use the debugger to verify your solution works for:
- R1=5, R2=10, R3=3 → R10=10
- R1=20, R2=15, R3=25 → R10=25

### Challenge 2: String Length

Write a program that:
1. Has a null-terminated string in memory starting at 0x1000
2. Counts characters until it finds 0x00
3. Stores length in R10

Use the debugger to step through and watch the counter increment.

### Challenge 3: Bubble Sort

Implement bubble sort for a 5-element array. Use the debugger to:
1. Set breakpoints at key points (swap, comparison)
2. Watch the array change in memory
3. Verify final sorted order

## Summary

In this chapter, you learned:

✅ Assembly language provides readable representation of machine code
✅ The assembler converts assembly → tokens → AST → machine code
✅ Labels provide symbolic names for addresses
✅ The debugger enables interactive program inspection
✅ Breakpoints pause execution at specific addresses
✅ State inspection shows registers, memory, and flags
✅ Stepping through programs helps find bugs

## Next Steps

In Chapter 6, we'll explore the **Operating System Kernel**: how the OS boots, manages processes, schedules CPU time, and provides system calls for programs to interact with the system.

## Further Reading

- `crates/vos-asm/src/lexer.rs` - Lexical analysis
- `crates/vos-asm/src/parser.rs` - Syntax parsing
- `crates/vos-asm/src/assembler.rs` - Code generation
- `crates/vos-debugger/src/debugger.rs` - Interactive debugging
- `examples/asm/` - Example assembly programs

## Testing Your Understanding

1. What's the difference between a label and a register?
2. Why does the assembler need two passes for some programs?
3. What happens when you set a breakpoint and run `continue`?
4. How can you tell if a program is in an infinite loop?
5. What's the benefit of labels over hard-coded addresses?

**Answers:**
1. A label is a symbolic name for an address; a register is storage
2. First pass collects labels, second pass resolves label references
3. Execution runs until PC reaches breakpoint address or program halts
4. Use debugger to step and watch PC/registers not changing meaningfully
5. Labels automatically adjust if code changes; hard-coded addresses break
