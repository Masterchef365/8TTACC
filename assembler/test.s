im_a_label:                     // labels can contain a-z,A-Z,_
5F -> LED                       // Literals count as sources (Operand)
00 -> PC.latch : if_1           // Executes if the "1" flag is set

lo@im_a_label -> PC.latch       // These two lines set the PC to im_a_label
hi@im_a_label -> PC       

55 -> RAM.low : if_carry | if_1  // Executes if both the if_carry and if_1 flags are set
FF -> RAM.high : if_1 | if_carry  // Executes if both the if_carry and if_1 flags are set

im_also_a_label:
lo@im_also_a_label -> PC.latch       // These two lines set the PC to im_a_label


//8F -> RAM                       // Assembler will throw an error!
//ACC.plus -> LED                 // Assembler will throw an error!
