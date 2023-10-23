use crate::instruction::Instruction;

#[derive(Debug, Clone, Copy)]
pub enum JumpFlag {
    Always,
    IfZero,
    IfNegative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicBlockId(pub usize);

#[derive(Debug)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub instructions: Vec<Instruction>,
    pub outgoing_jumps: Vec<(BasicBlockId, JumpFlag)>,
    pub incoming_jumps: Vec<(BasicBlockId, Option<JumpFlag>)>,
}


