use crate::instruction::Instruction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpFlag {
    Always,
    IfZero,
    IfNotZero,
    IfNegative,
    IfNotNegative,
    IfPositive,
    IfNotPositive,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BasicBlockId(pub usize);

#[derive(Debug)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub instructions: Vec<Instruction>,
    pub outgoing_jumps: Vec<(BasicBlockId, JumpFlag)>,
    pub incoming_jumps: Vec<(BasicBlockId, JumpFlag)>,
}


