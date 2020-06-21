# 8TTACC Toolchain
A toolchain for the 8TTACC 8-bit CMOS TTA Computer.

The project is split into a number of crates:
* `common/`: Common types used across the project
* `assembler/`: A two-pass assembler
* `emulator/`: A basic emulator

# Running
Get the Rust toolchain from https://rustup.rs/

Assembler: `cargo run --bin assember -- <source.s> <out.bin>`

Emulator: `cargo run --bin emulator -- <out.bin>`

# Syntax
Sources:
```
00: EXP
01: ACC
10: RAM
11: <literal in hex> (Special, loads next byte)
11: 'c' (Load character)
```

Destinations:
```
0000: RAM
0001: ACC.plus
0010: ACC.nand
0011: ACC
0100: PC.latch
0101: PC
0110: RAM.low
0111: RAM.high
1001: LED
1010: carry.set
1011: carry.reset
```

Conditions:
`if_1` : Executes if the "1" flag is set
`if_carry`: Executes if the carry flag is set

Move operator: `Source -> Destination`
Label: `this_is_a_label:`

Example:
```
ACC -> ACC                      // You won't need to type this, it'll always be there
im_a_label:                     // labels can contain a-z,A-Z,_
5F -> LED                       // Literals count as sources (Operand)
00 -> PC.latch : if_1           // Executes if the "1" flag is set
lo@im_a_label -> PC.latch       // These two lines set the PC to im_a_label
hi@im_a_label -> PC       
55 -> RAM.low : if_carry | if_1  // Executes if either of if_carry and if_1 flags are set
8F -> RAM                        // Assembler will throw an error!
ACC.plus -> LED                  // Assembler will throw an error!
```

# Macros
It is recommended to use the `nasm` preprocessor to expand macros for this language. To invoke the `nasm` preprocessor, simply pass your source file in with the `-E` argument:

`nasm -E <source.s> > preprocessed.s`

Or, if you wish to preprocess and assemble on a single line in Bash:

`cargo run --bin assembler -- <(nasm -E source.s) out.bin`

Documentation here:
* https://www.nasm.us/doc/nasmdoc4.html
* https://www.tortall.net/projects/yasm/manual/html/nasm-multi-line-macros.html

Example:
```
%macro jconst 0
hi@constant_label -> PC.latch
lo@constant_label -> PC
%endmacro

%macro jump 1
hi@%1 -> PC.latch
lo@%1 -> PC
%endmacro

%define MYCONST 03
MYCONST -> ACC

infinite_loop:
jump infinite_loop

constant_label:
jconst
```
