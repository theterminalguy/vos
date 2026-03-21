# Chapter 3: Memory Systems

## Learning Objectives

By the end of this chapter, you will:
- Understand physical vs. virtual memory
- Learn how the MMU translates virtual addresses to physical addresses
- Understand page-based memory management
- Learn about memory protection and permissions
- Write programs that interact with memory safely

## Introduction

Memory is where programs and data are stored. In modern systems, memory management is complex, involving virtual memory, paging, and protection mechanisms. VOS implements a simplified but functional memory system that demonstrates these core concepts.

## Memory Architecture Overview

VOS uses a three-layer memory architecture:

```
┌─────────────────────────────────────┐
│         CPU (vos-cpu)               │
│  Requests memory at virtual address │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│    MMU - Memory Management Unit     │
│  Translates virtual → physical addr │
│  Checks permissions                 │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│         RAM (Physical Memory)        │
│  Actual storage of bytes            │
└─────────────────────────────────────┘
```

## Memory Layout

VOS defines a standard memory map:

```
0x00000000 - 0x00000FFF  (4KB)     Interrupt Vector Table (IVT)
0x00001000 - 0x000FFFFF  (1020KB)  Kernel Code + Data
0x00100000 - 0x001FFFFF  (1MB)     Kernel Stack
0x00200000 - 0x3FFFFFFF  (~1GB)    User Space
0x40000000 - 0x7FFFFFFF  (1GB)     Heap
0x80000000 - 0xBFFFFFFF  (1GB)     Memory-Mapped I/O
0xC0000000 - 0xFFFFFFFF  (1GB)     Reserved
```

These regions are defined in `vos-core`:

```rust
pub mod memory_regions {
    use super::AddressRange;

    pub const IVT: AddressRange = AddressRange {
        start: 0x0000_0000,
        end: 0x0000_1000,
    };

    pub const KERNEL_CODE: AddressRange = AddressRange {
        start: 0x0000_1000,
        end: 0x0010_0000,
    };

    pub const USER_SPACE: AddressRange = AddressRange {
        start: 0x0020_0000,
        end: 0x4000_0000,
    };

    pub const HEAP: AddressRange = AddressRange {
        start: 0x4000_0000,
        end: 0x8000_0000,
    };

    pub const MMIO: AddressRange = AddressRange {
        start: 0x8000_0000,
        end: 0xC000_0000,
    };
}
```

## Physical Memory (RAM)

The RAM module provides raw byte storage:

```rust
pub struct Ram {
    data: Vec<Byte>,
    size: usize,
}

impl Ram {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            size,
        }
    }

    pub fn read_byte(&self, address: Address) -> Result<Byte> {
        if address as usize >= self.size {
            return Err(MemoryError::OutOfBounds {
                address,
                size: self.size,
            }
            .into());
        }
        Ok(self.data[address as usize])
    }

    pub fn write_byte(&mut self, address: Address, value: Byte) -> Result<()> {
        if address as usize >= self.size {
            return Err(MemoryError::OutOfBounds {
                address,
                size: self.size,
            }
            .into());
        }
        self.data[address as usize] = value;
        Ok(())
    }
}
```

### Word vs. Byte Access

Memory can be accessed at byte or word (4-byte) granularity:

**Byte Access:**
```rust
ram.write_byte(0x1000, 0x42)?;  // Write single byte
let byte = ram.read_byte(0x1000)?;
```

**Word Access (Little-Endian):**
```rust
ram.write_word(0x1000, 0x12345678)?;
// Memory layout: [0x78, 0x56, 0x34, 0x12]

let word = ram.read_word(0x1000)?;  // 0x12345678
```

VOS uses **little-endian** byte order: the least significant byte is stored at the lowest address.

## Virtual Memory and the MMU

The MMU (Memory Management Unit) translates virtual addresses to physical addresses:

### Why Virtual Memory?

1. **Isolation**: Each process has its own address space
2. **Protection**: Prevent processes from accessing each other's memory
3. **Flexibility**: Programs can be loaded anywhere in physical memory
4. **Memory Management**: Enable features like demand paging

### Page-Based Memory

VOS uses **4KB pages** (4096 bytes):

```rust
pub const PAGE_SIZE: usize = 4096;  // 4KB pages

pub fn page_number(address: Address) -> PageNumber {
    (address / PAGE_SIZE as u32) as PageNumber
}

pub fn page_offset(address: Address) -> u32 {
    address % PAGE_SIZE as u32
}
```

A virtual address is split into two parts:

```
Virtual Address (32 bits)
┌──────────────────┬──────────────┐
│  Page Number     │ Page Offset  │
│  (bits 31-12)    │ (bits 11-0)  │
│  20 bits         │ 12 bits      │
└──────────────────┴──────────────┘
      |                    |
      |                    └─→ Offset within page (0-4095)
      └─→ Which page (0-1,048,575)
```

**Example:**
```
Virtual Address: 0x00012345
Page Number:     0x00012 (page 18)
Page Offset:     0x345   (offset 837 within page)
```

### Page Table Entries

Each page has a Page Table Entry (PTE) that describes it:

```rust
pub struct PageTableEntry {
    /// Physical frame number
    pub frame: FrameNumber,

    /// Page is present in memory
    pub present: bool,

    /// Page is writable
    pub writable: bool,

    /// Page is executable
    pub executable: bool,

    /// Page has been accessed
    pub accessed: bool,

    /// Page has been modified
    pub dirty: bool,
}
```

### Address Translation

The MMU translates virtual → physical addresses:

```rust
pub fn translate(&mut self, virtual_address: Address, write: bool) -> Result<Address> {
    if !self.paging_enabled {
        return Ok(virtual_address);  // Direct mapping
    }

    // Extract page number and offset
    let page = page_number(virtual_address);
    let offset = page_offset(virtual_address);

    // Look up page table entry
    let entry = self.page_table.lookup_mut(page)?;

    // Check if page is present
    if !entry.present {
        return Err(MemoryError::PageFault {
            address: virtual_address,
            present: false,
            write,
        }
        .into());
    }

    // Check write permission
    if write && !entry.writable {
        return Err(MemoryError::ReadOnly(virtual_address).into());
    }

    // Mark as accessed (and dirty if writing)
    entry.accessed = true;
    if write {
        entry.dirty = true;
    }

    // Calculate physical address
    let physical_address = (entry.frame as usize * PAGE_SIZE) as Address + offset;

    Ok(physical_address)
}
```

**Translation Example:**

```
Virtual Address: 0x00012345

Step 1: Extract page and offset
  Page Number:  0x12 (18)
  Page Offset:  0x345 (837)

Step 2: Lookup page table
  Entry[18].frame = 5
  Entry[18].present = true
  Entry[18].writable = true

Step 3: Calculate physical address
  Physical Frame: 5
  Frame Address:  5 * 4096 = 0x5000
  Physical Addr:  0x5000 + 0x345 = 0x5345

Virtual 0x00012345 → Physical 0x00005345
```

## Memory Operations

### Identity Mapping

For simple scenarios, you can map virtual addresses directly to physical:

```rust
// Map virtual pages 0-255 to physical frames 0-255
for page in 0..256 {
    mmu.identity_map(
        page,
        PagePermissions {
            writable: true,
            executable: true,
        },
    )?;
}
```

### Custom Mapping

Map specific virtual pages to any physical frame:

```rust
// Map virtual page 100 to physical frame 50
mmu.map_page(
    100,  // virtual page
    50,   // physical frame
    PagePermissions {
        writable: true,
        executable: false,
    },
)?;
```

### Read-Only Pages

Protect code or constant data:

```rust
// Map kernel code as read-only, executable
mmu.map_page(
    kernel_page,
    frame,
    PagePermissions {
        writable: false,  // Read-only
        executable: true, // Can execute
    },
)?;
```

## Memory Module Integration

The `Memory` struct combines RAM and MMU:

```rust
pub struct Memory {
    ram: Ram,
    mmu: Mmu,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self {
            ram: Ram::new(size),
            mmu: Mmu::new(),
        }
    }

    pub fn read_word(&mut self, address: Address) -> Result<Word> {
        // Translate virtual → physical
        let physical = self.mmu.translate(address, false)?;

        // Access physical RAM
        self.ram.read_word(physical)
    }

    pub fn write_word(&mut self, address: Address, value: Word) -> Result<()> {
        // Translate virtual → physical
        let physical = self.mmu.translate(address, true)?;

        // Access physical RAM
        self.ram.write_word(physical, value)
    }
}
```

## Memory Protection

### Page Faults

A page fault occurs when:
- Page is not present (`present = false`)
- Write to read-only page (`writable = false`)
- Execute from non-executable page (`executable = false`)

```rust
// Try to write to read-only page
match memory.write_word(ro_address, value) {
    Err(VosError::Memory(MemoryError::ReadOnly(addr))) => {
        println!("Cannot write to read-only page at 0x{:08X}", addr);
    }
    _ => {}
}
```

### Bounds Checking

RAM enforces bounds checking:

```rust
// Try to access beyond RAM size
match ram.read_byte(0xFFFFFFFF) {
    Err(VosError::Memory(MemoryError::OutOfBounds { address, size })) => {
        println!("Address 0x{:08X} exceeds RAM size 0x{:08X}", address, size);
    }
    _ => {}
}
```

## Example: Protected Memory Regions

Let's set up memory with different protection levels:

```assembly
; Kernel code (read-only, executable)
start:
    LUI  R1, 0x8000         ; Load I/O base address
    ADDI R2, R0, 72         ; R2 = 'H'
    SW   R2, R1, 0          ; Write to display
    HALT

; This would cause a fault:
; SW R3, R0, 0x1000        ; Try to write to kernel code (read-only)
```

Setting up the memory map:

```rust
let mut memory = Memory::new(16 * 1024 * 1024);  // 16MB

// Map kernel code as read-only, executable (pages 0-255)
for page in 0..256 {
    memory.mmu_mut().map_page(
        page,
        page,
        PagePermissions {
            writable: false,   // Read-only
            executable: true,  // Can execute
        },
    )?;
}

// Map user space as read-write, non-executable (pages 512-1023)
for page in 512..1024 {
    memory.mmu_mut().map_page(
        page,
        page,
        PagePermissions {
            writable: true,    // Read-write
            executable: false, // Cannot execute (security)
        },
    )?;
}
```

## Hands-On Exercise: Stack Implementation

Implement a simple stack in memory.

**Requirements:**
- Stack grows downward from 0x00200000
- Use R15 as stack pointer
- Implement push and pop operations

**Solution:**

```assembly
; Initialize stack pointer
LUI  R15, 0x0020         ; R15 = 0x00200000 (stack base)

; Push 42 onto stack
ADDI R1, R0, 42          ; R1 = 42
SUBI R15, R15, 4         ; SP -= 4 (allocate space)
SW   R1, R15, 0          ; Memory[SP] = R1

; Push 99 onto stack
ADDI R2, R0, 99          ; R2 = 99
SUBI R15, R15, 4         ; SP -= 4
SW   R2, R15, 0          ; Memory[SP] = R2

; Pop from stack into R3
LW   R3, R15, 0          ; R3 = Memory[SP] (99)
ADDI R15, R15, 4         ; SP += 4 (deallocate)

; Pop from stack into R4
LW   R4, R15, 0          ; R4 = Memory[SP] (42)
ADDI R15, R15, 4         ; SP += 4

HALT
; R3 = 99, R4 = 42
```

## Code Walkthrough: Memory Implementation

### Loading Programs

The `load()` method writes program data directly to physical memory:

```rust
impl Memory {
    pub fn load(&mut self, address: Address, data: &[u8]) -> Result<()> {
        // Bypass MMU for loading
        self.ram.load(address, data)
    }
}

impl Ram {
    pub fn load(&mut self, address: Address, data: &[u8]) -> Result<()> {
        let start = address as usize;
        let end = start + data.len();

        if end > self.size {
            return Err(MemoryError::OutOfBounds {
                address,
                size: self.size,
            }
            .into());
        }

        self.data[start..end].copy_from_slice(data);
        Ok(())
    }
}
```

### Memory Dumps

For debugging, memory can be displayed as hex:

```rust
impl Ram {
    pub fn hexdump(&self, start: Address, count: usize) -> String {
        let mut output = String::new();

        for i in 0..count {
            let addr = start + (i as u32 * 16);

            // Address
            output.push_str(&format!("0x{:08X}: ", addr));

            // Hex bytes
            for j in 0..16 {
                if let Ok(byte) = self.read_byte(addr + j) {
                    output.push_str(&format!("{:02X} ", byte));
                } else {
                    output.push_str("   ");
                }
            }

            // ASCII representation
            output.push_str(" |");
            for j in 0..16 {
                if let Ok(byte) = self.read_byte(addr + j) {
                    let ch = if (32..=126).contains(&byte) {
                        byte as char
                    } else {
                        '.'
                    };
                    output.push(ch);
                }
            }
            output.push_str("|\n");
        }

        output
    }
}
```

**Example output:**
```
0x00001000: 04 00 40 00 05 00 50 00  00 11 12 23 80 00 00 00  |..@...P....#....|
0x00001010: 00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|
```

## Challenge Problems

### Challenge 1: Page Table Walker

Write code to display all mapped pages in the page table, showing:
- Virtual page number
- Physical frame number
- Permissions (R/W/X)

### Challenge 2: Memory Copy

Write an assembly program that copies 16 bytes from address 0x1000 to 0x2000.

**Hint:** Use a loop with LB (load byte) and SB (store byte).

### Challenge 3: Protected Code

Set up a memory region where:
- Pages 0-9: Kernel code (RO, executable)
- Pages 10-19: Kernel data (RW, non-executable)
- Pages 20-29: User code (RO, executable)
- Pages 30-39: User data (RW, non-executable)

Write tests that verify the protection works.

## Summary

In this chapter, you learned:

✅ Physical memory (RAM) provides raw byte storage
✅ Virtual memory uses pages (4KB) for management
✅ The MMU translates virtual addresses to physical addresses
✅ Page table entries control permissions (read/write/execute)
✅ Memory protection prevents unauthorized access
✅ VOS uses a standard memory layout with defined regions

## Next Steps

In Chapter 4, we'll explore **I/O devices**: how the CPU communicates with external hardware like displays, keyboards, and timers through memory-mapped I/O.

## Further Reading

- `crates/vos-memory/src/ram.rs` - Physical memory implementation
- `crates/vos-memory/src/mmu.rs` - MMU and paging
- `crates/vos-memory/src/memory.rs` - Integrated memory subsystem
- `crates/vos-core/src/types.rs` - Memory regions and address types

## Testing Your Understanding

1. Why does VOS use 4KB pages instead of 1KB or 16KB?
2. What happens if you try to execute code from a non-executable page?
3. How many pages can be addressed with a 20-bit page number?
4. Why is little-endian used for word storage?
5. What's the difference between virtual and physical addresses?

**Answers:**
1. 4KB is a common page size balancing overhead and flexibility
2. A page fault occurs (execution protection violation)
3. 2^20 = 1,048,576 pages (4GB address space / 4KB pages)
4. Little-endian is common on x86 and simplifies multi-byte operations
5. Virtual addresses are what programs see; physical addresses are actual RAM locations
