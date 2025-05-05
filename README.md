# Hack Assembler

A Rust implementation of an assembler for the Hack assembly language, which translates Hack assembly code (.asm files) into Hack machine code (.hack files). This assembler is part of the toolchain for the Hack computer architecture.

## Overview

The Hack assembler converts symbolic Hack assembly language into binary machine code that can be executed by the Hack hardware platform. It handles:

- Symbol resolution (variables and labels)
- Translation of A-instructions (@value)
- Translation of C-instructions (dest=comp;jump)
- Predefined symbols (R0-R15, SCREEN, KBD, SP, LCL, ARG, THIS, THAT)

## Features

- Removes comments and whitespace from assembly code
- Handles both symbolic and numeric addressing
- Supports all standard Hack assembly instructions
- Manages label definitions (XXX) and variables
- Automatically assigns memory addresses for variables
- Converts instructions to 16-bit binary machine code

## Usage

```bash
assembler <file_name>.asm
```

The assembler will create an output file with the same name but with a .hack extension.

### Command Line Options

- `-h`: Display help message

## Instruction Format

### A-Instructions

Format: `@value`

- `value` can be either a non-negative decimal number or a symbol
- Translated to: `0vvvvvvvvvvvvvvv` (16 bits where v is the binary value)

### C-Instructions

Format: `dest=comp;jump`

- `dest` and `jump` are optional
- Translated to: `111accccccdddjjj`
  - a: Determines whether to use A or M register
  - c: Computation bits
  - d: Destination bits
  - j: Jump bits

## Symbol Tables

### Predefined Symbols

- R0-R15: RAM addresses 0-15
- SCREEN: RAM address 16384
- KBD: RAM address 24576
- SP, LCL, ARG, THIS, THAT: RAM addresses 0-4

### Label Symbols

- Declared using (XXX)
- References the instruction memory location following the declaration

### Variable Symbols

- Created when a new symbol is encountered in an A-instruction
- Allocated to consecutive memory locations starting from RAM address 16

## Implementation Details

The assembler processes the input file in two passes:

1. First Pass:

   - Removes comments and whitespace
   - Processes label declarations
   - Builds symbol table for labels

2. Second Pass:
   - Processes A-instructions and C-instructions
   - Allocates variables to memory
   - Generates binary machine code

## Example

Input (.asm):

```
// Adds 1 + 2
@1
D=A
@2
D=D+A
@3
M=D
```

Output (.hack):

```
0000000000000001
1110110000010000
0000000000000010
1110000010010000
0000000000000011
1110001100001000
```

## Dependencies

- lazy_static: For static initialization of lookup tables
- regex: For comment and whitespace removal

## Building

```bash
cargo build --release
```

## License

This project is part of the open-source implementation of the Hack computer system.
