pub type Label = String;

#[derive(Debug, PartialEq, Clone)]
pub enum Source {
    Expansion,
    Accumulator,
    Memory,
    Operand(u8),
    LabelLo(Label),
    LabelHi(Label),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Destination {
    Memory,
    MemAddressLo,
    MemAddressHi,
    Accumulator,
    AccumulatorPlus,
    AccumulatorNand,
    ProgramCounter,
    ProgramCounterLatch,
    Led,
    CarrySet,
    CarryReset,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Operation {
    pub src: Source,
    pub dest: Destination,
    pub cond_1: bool,
    pub cond_carry: bool,
}
