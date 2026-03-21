# Chapter 4: I/O Devices and Hardware Integration

## Learning Objectives

By the end of this chapter, you will:
- Understand memory-mapped I/O
- Learn how to interact with display, keyboard, and timer devices
- Build a complete virtual machine integrating CPU, memory, and I/O
- Write programs that use I/O devices
- Understand interrupt-driven I/O concepts

## Introduction

I/O (Input/Output) devices allow the computer to interact with the outside world. In VOS, devices like the display, keyboard, and timer are accessed through **memory-mapped I/O** - they appear as special memory addresses that the CPU can read and write.

## Memory-Mapped I/O

Instead of special I/O instructions, VOS uses memory-mapped I/O:

```
Memory Map:
0x80000000 - 0x80000F9F   Display (4000 bytes)
0x80002000 - 0x8000200F   Keyboard (16 bytes)
0x80003000 - 0x8000300F   Timer (16 bytes)
```

When the CPU reads or writes these addresses, the **MemoryBus** routes the access to the appropriate device instead of RAM.

### Advantages of Memory-Mapped I/O

1. **Simplicity**: Use regular load/store instructions
2. **Consistency**: Same programming model as memory
3. **Flexibility**: Easy to add new devices
4. **Protection**: Memory protection applies to I/O

## The Device Trait

All I/O devices implement a common interface:

```rust
pub trait Device {
    /// Read a byte from device at offset
    fn read_byte(&mut self, offset: u32) -> Result<Byte>;

    /// Write a byte to device at offset
    fn write_byte(&mut self, offset: u32, value: Byte) -> Result<()>;

    /// Read a 32-bit word (default: uses read_byte)
    fn read_word(&mut self, offset: u32) -> Result<Word> {
        let b0 = self.read_byte(offset)? as Word;
        let b1 = self.read_byte(offset + 1)? as Word;
        let b2 = self.read_byte(offset + 2)? as Word;
        let b3 = self.read_byte(offset + 3)? as Word;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    /// Write a 32-bit word (default: uses write_byte)
    fn write_word(&mut self, offset: u32, value: Word) -> Result<()> {
        self.write_byte(offset, (value & 0xFF) as Byte)?;
        self.write_byte(offset + 1, ((value >> 8) & 0xFF) as Byte)?;
        self.write_byte(offset + 2, ((value >> 16) & 0xFF) as Byte)?;
        self.write_byte(offset + 3, ((value >> 24) & 0xFF) as Byte)?;
        Ok(())
    }

    /// Get device base address
    fn base_address(&self) -> Address;

    /// Get device size in bytes
    fn size(&self) -> usize;

    /// Get device name
    fn name(&self) -> &str;

    /// Optional: Tick for time-based updates
    fn tick(&mut self, _cycles: u64) {}

    /// Optional: Reset device
    fn reset(&mut self) {}
}
```

## Display Device

The display is an 80x25 text-mode screen:

### Display Specifications

```
Base Address: 0x80000000
Size:         4000 bytes (80 × 25 × 2)
Format:       [character, attribute, character, attribute, ...]
```

Each character cell uses 2 bytes:
- **Byte 0**: ASCII character code
- **Byte 1**: Attribute (foreground/background color)

### Display Structure

```rust
pub struct Display {
    buffer: Vec<CharCell>,  // 80x25 grid
    cursor: usize,          // Current cursor position
}

pub struct CharCell {
    pub character: u8,  // ASCII code
    pub attribute: u8,  // Color attributes
}
```

### Writing to the Display

**Method 1: Direct Memory Writes**

```assembly
; Display base address
LUI  R1, 0x8000          ; R1 = 0x80000000

; Write 'H' at position 0
ADDI R2, R0, 72          ; R2 = 'H' (ASCII 72)
SB   R2, R1, 0           ; Display[0] = 'H'

; Write attribute (white on black)
ADDI R3, R0, 0x07        ; R3 = 0x07 (light gray on black)
SB   R3, R1, 1           ; Display[1] = attribute
```

**Method 2: Using put_char() API**

```rust
display.put_char(b'H');  // Writes 'H' and advances cursor
display.put_char(b'e');
display.put_char(b'l');
display.put_char(b'l');
display.put_char(b'o');
display.put_char(b'\n'); // Newline moves to next line
```

### Display Memory Layout

```
Offset   Contents
------   --------
0x0000   Character at row 0, col 0
0x0001   Attribute at row 0, col 0
0x0002   Character at row 0, col 1
0x0003   Attribute at row 0, col 1
...
0x009E   Character at row 0, col 79
0x009F   Attribute at row 0, col 79
0x00A0   Character at row 1, col 0 (next line)
```

### Scrolling

When the cursor reaches the end of the screen, the display automatically scrolls:

```rust
fn scroll(&mut self) {
    // Copy lines 1-24 to 0-23
    self.buffer.copy_within(DISPLAY_WIDTH..DISPLAY_SIZE, 0);

    // Clear last line
    let last_line_start = DISPLAY_SIZE - DISPLAY_WIDTH;
    for i in last_line_start..DISPLAY_SIZE {
        self.buffer[i] = CharCell::default();
    }
}
```

## Keyboard Device

The keyboard provides character input through a FIFO buffer:

### Keyboard Specifications

```
Base Address: 0x80002000
Size:         16 bytes
Registers:
  0x00: DATA     - Read character from buffer
  0x04: STATUS   - Status flags (bit 0 = data available)
  0x08: CONTROL  - Control flags (bit 0 = enable interrupts)
  0x0C: Reserved
```

### Keyboard Structure

```rust
pub struct Keyboard {
    buffer: VecDeque<u8>,  // FIFO buffer (max 256 chars)
    control: u8,            // Control register
    interrupt_enabled: bool,
}
```

### Reading from Keyboard

**Polling (wait for key):**

```assembly
; Keyboard base
LUI  R1, 0x8000          ; R1 = 0x80000000
ADDI R1, R1, 0x2000      ; R1 = 0x80002000 (keyboard base)

wait_key:
    LW   R2, R1, 4       ; Read STATUS register
    ANDI R3, R2, 0x01    ; Check bit 0 (data available)
    BEQ  R3, R0, wait_key ; Loop if no data

    ; Data available, read it
    LB   R4, R1, 0       ; Read DATA register
    ; R4 now contains the key pressed
```

**Checking for input (non-blocking):**

```rust
if keyboard.has_data() {
    let key = keyboard.pop_key().unwrap();
    println!("Key pressed: {}", key as char);
}
```

### Keyboard Status Register

```
Bit 0: DATA_READY - Set if character available
Bit 1: BUFFER_FULL - Set if buffer is full (256 chars)
Bit 7: INTERRUPT - Set if interrupt should be triggered
```

## Timer Device

The timer provides periodic interrupts for time-based operations:

### Timer Specifications

```
Base Address: 0x80003000
Size:         16 bytes
Registers:
  0x00: COUNTER  - Current counter value (read-only, counts down)
  0x04: RELOAD   - Reload value (read/write)
  0x08: CONTROL  - Control flags (bit 0 = enable, bit 1 = interrupt)
  0x0C: STATUS   - Status flags (bit 0 = expired)
```

### Timer Structure

```rust
pub struct Timer {
    counter: u32,       // Current count
    reload: u32,        // Value to reload when counter reaches 0
    control: u8,        // Control flags
    status: u8,         // Status flags
    total_cycles: u64,  // Total cycles elapsed
}
```

### Using the Timer

**Set up a periodic timer:**

```assembly
; Timer base
LUI  R1, 0x8000
ADDI R1, R1, 0x3000      ; R1 = 0x80003000 (timer base)

; Set reload value to 1000
LUI  R2, 0x0000
ADDI R2, R2, 1000
SW   R2, R1, 4           ; RELOAD = 1000

; Enable timer with interrupts
ADDI R3, R0, 0x03        ; Enable + Interrupt
SB   R3, R1, 8           ; CONTROL = 0x03
```

**Check if timer expired:**

```assembly
check_timer:
    LB   R4, R1, 12      ; Read STATUS
    ANDI R5, R4, 0x01    ; Check EXPIRED bit
    BEQ  R5, R0, check_timer

    ; Timer expired, clear status
    SB   R5, R1, 12      ; Write STATUS to clear
```

### Timer Operation

```rust
fn tick_internal(&mut self, cycles: u64) {
    if !self.is_enabled() {
        return;
    }

    for _ in 0..cycles {
        if self.counter > 0 {
            self.counter -= 1;

            if self.counter == 0 {
                // Timer expired
                self.status |= STATUS_EXPIRED;

                // Reload counter for next period
                self.counter = self.reload;

                // Trigger interrupt if enabled
                if self.interrupt_enabled() {
                    // (Interrupt handling covered in future chapters)
                }
            }
        }
    }
}
```

## Virtual Machine Integration

The `VirtualMachine` struct combines all components:

```rust
pub struct VirtualMachine {
    cpu: Cpu,
    memory: Memory,
    display: Display,
    keyboard: Keyboard,
    timer: Timer,
    cycles: u64,
}
```

### Memory Bus

The `MemoryBus` routes memory accesses to devices or RAM:

```rust
struct MemoryBus<'a> {
    memory: &'a mut Memory,
    display: &'a mut Display,
    keyboard: &'a mut Keyboard,
    timer: &'a mut Timer,
}

impl<'a> MemoryBus<'a> {
    fn route_device(&mut self, address: Address) -> Option<&mut dyn Device> {
        // Check if address is in I/O region
        if address >= DISPLAY_BASE && address < DISPLAY_BASE + DISPLAY_SIZE as u32 {
            Some(self.display)
        } else if address >= KEYBOARD_BASE && address < KEYBOARD_BASE + KEYBOARD_SIZE as u32 {
            Some(self.keyboard)
        } else if address >= TIMER_BASE && address < TIMER_BASE + TIMER_SIZE as u32 {
            Some(self.timer)
        } else {
            None  // Regular memory
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
}
```

### Running the Virtual Machine

```rust
impl VirtualMachine {
    pub fn step(&mut self) -> Result<bool> {
        // Execute one CPU instruction with I/O access
        let should_continue = self.cpu.step(&mut MemoryBus {
            memory: &mut self.memory,
            display: &mut self.display,
            keyboard: &mut self.keyboard,
            timer: &mut self.timer,
        })?;

        // Update devices
        self.timer.tick(1);
        self.cycles += 1;

        Ok(should_continue)
    }

    pub fn run(&mut self) -> Result<()> {
        while self.step()? {
            // Continue execution
        }
        Ok(())
    }
}
```

## Example: Hello World Program

Complete program that writes "Hello" to the display:

```assembly
; Hello World
; Writes "Hello" to display at 0x80000000

start:
    ; Load display base address
    LUI  R1, 0x8000         ; R1 = 0x80000000

    ; Write 'H' (0x48)
    ADDI R2, R0, 72
    SB   R2, R1, 0          ; Display[0] = 'H'

    ; Write 'e' (0x65)
    ADDI R2, R0, 101
    SB   R2, R1, 2          ; Display[2] = 'e'

    ; Write 'l' (0x6C)
    ADDI R2, R0, 108
    SB   R2, R1, 4          ; Display[4] = 'l'

    ; Write second 'l'
    SB   R2, R1, 6          ; Display[6] = 'l'

    ; Write 'o' (0x6F)
    ADDI R2, R0, 111
    SB   R2, R1, 8          ; Display[8] = 'o'

    HALT
```

Running this program:

```rust
use vos_hardware::VirtualMachine;
use vos_asm::assemble;

let source = r#"
    LUI  R1, 0x8000
    ADDI R2, R0, 72
    SB   R2, R1, 0
    ADDI R2, R0, 101
    SB   R2, R1, 2
    ADDI R2, R0, 108
    SB   R2, R1, 4
    SB   R2, R1, 6
    ADDI R2, R0, 111
    SB   R2, R1, 8
    HALT
"#;

let machine_code = assemble(source)?;

let mut vm = VirtualMachine::new(1024 * 1024);
vm.load_program(0, &machine_code)?;
vm.cpu_mut().set_pc(0);
vm.run()?;

// Get display output
let output = vm.display_output();
assert!(output.starts_with("Hello"));
```

## Hands-On Exercise: Echo Program

Write a program that reads a key from the keyboard and displays it on the screen.

**Requirements:**
- Wait for keyboard input
- Read the character
- Write it to the display
- Repeat until 'Q' is pressed

**Solution:**

```assembly
start:
    ; Load device base addresses
    LUI  R10, 0x8000        ; Display base
    LUI  R11, 0x8000
    ADDI R11, R11, 0x2000   ; Keyboard base

    ADDI R12, R0, 0         ; Display offset

loop:
    ; Wait for keyboard input
wait:
    LW   R1, R11, 4         ; Read keyboard STATUS
    ANDI R2, R1, 0x01       ; Check DATA_READY bit
    BEQ  R2, R0, wait       ; Loop if no data

    ; Read character
    LB   R3, R11, 0         ; Read DATA

    ; Check for 'Q' (0x51)
    ADDI R4, R0, 81
    BEQ  R3, R4, done       ; Exit if Q pressed

    ; Write to display
    ADD  R5, R10, R12       ; Calculate display address
    SB   R3, R5, 0          ; Write character

    ; Advance display offset
    ADDI R12, R12, 2        ; Move to next character cell

    ; Check if end of line (160 bytes = 80 chars)
    ADDI R6, R0, 160
    BLT  R12, R6, loop      ; Continue if not at end

    ; Wrap to beginning
    ADDI R12, R0, 0
    J    loop

done:
    HALT
```

## Code Walkthrough: Device Implementation

### Display put_char Implementation

```rust
pub fn put_char(&mut self, character: u8) {
    if character == b'\n' {
        // Newline: move to start of next line
        self.cursor = ((self.cursor / DISPLAY_WIDTH) + 1) * DISPLAY_WIDTH;
    } else {
        // Write character at cursor
        self.buffer[self.cursor] = CharCell::with_char(character);
        self.cursor += 1;
    }

    // Scroll if needed
    if self.cursor >= DISPLAY_SIZE {
        self.scroll();
        self.cursor = DISPLAY_SIZE - DISPLAY_WIDTH;
    }
}
```

### Keyboard push_key Implementation

```rust
pub fn push_key(&mut self, key: u8) {
    if self.buffer.len() < 256 {
        self.buffer.push_back(key);

        // Trigger interrupt if enabled
        if self.interrupt_enabled {
            self.control |= CTRL_INTERRUPT_PENDING;
        }
    }
}

pub fn pop_key(&mut self) -> Option<u8> {
    self.buffer.pop_front()
}

pub fn has_data(&self) -> bool {
    !self.buffer.is_empty()
}
```

## Challenge Problems

### Challenge 1: Screen Clear

Write a program that fills the entire display with spaces, effectively clearing the screen.

**Hint:** Loop through all 2000 character positions (80×25).

### Challenge 2: Countdown Timer Display

Write a program that:
1. Sets the timer to count down from 10
2. Displays the current count on screen
3. Updates every time the timer expires
4. Halts when count reaches 0

### Challenge 3: Simple Line Editor

Implement a simple line editor that:
- Displays characters as typed
- Handles backspace (ASCII 8) to delete last character
- Displays newline when Enter (ASCII 13) is pressed
- Stops after 3 lines

## Summary

In this chapter, you learned:

✅ Memory-mapped I/O provides a unified interface for devices
✅ The Display device provides 80×25 text output
✅ The Keyboard device provides buffered character input
✅ The Timer device enables periodic events
✅ The VirtualMachine integrates CPU, memory, and all I/O devices
✅ The MemoryBus routes accesses to devices or RAM

## Next Steps

In Chapter 5, we'll explore the **Assembler and Debugger**: tools for writing, assembling, and debugging VOS programs interactively.

## Further Reading

- `crates/vos-io/src/display.rs` - Display device implementation
- `crates/vos-io/src/keyboard.rs` - Keyboard device implementation
- `crates/vos-io/src/timer.rs` - Timer device implementation
- `crates/vos-hardware/src/machine.rs` - Virtual machine integration

## Testing Your Understanding

1. Why use memory-mapped I/O instead of special I/O instructions?
2. How many characters can the display show at once?
3. What happens when the keyboard buffer is full?
4. How does the timer know when to trigger an interrupt?
5. Why does each display character use 2 bytes?

**Answers:**
1. Simplicity, consistency, and reuse of memory protection mechanisms
2. 2000 characters (80 columns × 25 rows)
3. New keys are dropped until buffer space is available
4. When counter reaches 0 and interrupt bit is set in CONTROL
5. One byte for character code, one byte for display attributes (colors)
