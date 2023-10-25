use crate::instruction::Instruction;

use super::jump_flag::JumpFlag;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BasicBlockId(pub usize);

#[derive(Debug)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub instructions: Vec<Instruction>,
    pub outgoing_jumps: Vec<(BasicBlockId, JumpFlag)>,
    pub incoming_jumps: Vec<(BasicBlockId, JumpFlag)>,
}

