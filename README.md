# 8TTACC Toolchain
An assembler toolchain for the 8TTACC Computer.

The project is split into a number of crates:
* `common/`: Common types used across the project
* `assembler/`: A two-pass assembler
* `emulator/`: A WIP emulator (and decompiler)

# Running:
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
55 -> RAM.low : if_carry | if_1  // Executes if both the if_carry and if_1 flags are set
8F -> RAM                        // Assembler will throw an error!
ACC.plus -> LED                  // Assembler will throw an error!
```
Later we may also introduce macros, like so:
```
def set_pc(label):
    lo@$label -> PC.latch
    hi@$label -> PC
    end

infinite_loop:
set_pc(infinite_loop)
```
