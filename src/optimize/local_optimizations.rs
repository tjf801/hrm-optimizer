use crate::{optimize::basic_blocks::BasicBlock, instruction::Address};

use super::basic_blocks::JumpFlag;
use super::control_flow_graph::{ProgramControlFlowGraph, Optimization};

pub fn local_optimization<T: FnMut(&mut BasicBlock) -> bool>(mut block_optimization: T) -> impl Optimization {
    move |graph: &mut ProgramControlFlowGraph| {
        let mut modified = false;
        for block in graph.blocks.iter_mut() {
            modified |= block_optimization(block);
        }
        modified
    }
}

pub fn simplify_outgoing_jumps(block: &mut BasicBlock) -> bool {
    match &mut block.outgoing_jumps[..] {
        [(_, flag1), (_, flag2)] if *flag2 == JumpFlag::Always => {
            let new_flag = match flag1 {
                JumpFlag::Always => None,
                JumpFlag::IfZero => Some(JumpFlag::IfNotZero),
                JumpFlag::IfNotZero => Some(JumpFlag::IfZero),
                JumpFlag::IfNegative => Some(JumpFlag::IfNotNegative),
                JumpFlag::IfNotNegative => Some(JumpFlag::IfNegative),
                JumpFlag::IfPositive => Some(JumpFlag::IfNotPositive),
                JumpFlag::IfNotPositive => Some(JumpFlag::IfPositive),
            };
            match new_flag {
                Some(flag) => { *flag2 = flag; },
                None => { block.outgoing_jumps.pop(); }
            }
            true
        }
        _ => { false }
    }
}

pub fn peephole_optimizations(block: &mut BasicBlock) -> bool {
    use crate::instruction::Instruction::*;
    
    let mut to_remove = Vec::new();
    
    for (i, instrs) in block.instructions.windows(2).enumerate() {
        match instrs {
            [ // statically detectable undefined behavior
                Outbox,
                Outbox | CopyTo(_) | Add(_) | Sub(_), // TODO: jumpz and jumpn?
            ] => {
                eprintln!(
                    "Undefined behavior warning! An OUTBOX followed by a {instr:?} instruction will always raise a EmptyHands error.", instr = instrs[1]);
            },
            [ // redundant accumulator instructions that immediately get overwritten
                CopyFrom(_) | Add(_) | Sub(_),
                CopyFrom(_) | BumpUp(_) | BumpDn(_),
            ] => {
                to_remove.push(i);
            },
            [ // optimize redundant COPYFROM after writing to the same address
                CopyTo(Address::Direct(a)) | BumpUp(Address::Direct(a)) | BumpDn(Address::Direct(a)),
                CopyFrom(Address::Direct(b))
            ] if a == b => {
                // NOTE: the reason you can only optimize this for direct addresses is because of the following:
                // ```hrm
                // -- start holding some number a, mem[3] is 3
                //     COPYTO   [3]    -- mem[3] is now Datacube(a)
                //     COPYFROM [3]    -- accumulator is now mem[a], not a
                // ```
                // The fact that it cannot be proven that any given indirect tile address does not point to
                // itself means that we cannot optimize out the COPYFROM instruction in that case.
                // A nearly identical argument also holds for the BUMPUP and BUMPDN cases.
                to_remove.push(i+1);
            },
            [Add(a), Sub(b)] | [Sub(a), Add(b)]
            if a == b => { // adding and subtracting the same number
                to_remove.push(i);
                to_remove.push(i+1);
            },
            [BumpUp(Address::Direct(a)), BumpDn(Address::Direct(b))] |
            [BumpDn(Address::Direct(a)), BumpUp(Address::Direct(b))]
            if a == b => { // bumping up and down the same address
                // NOTE: only direct addresses apply here for the same reason as above
                to_remove.push(i);
                to_remove.push(i+1);
            }
            _ => {},
        }
    }
    
    // make sure to remove duplicates (even if unlikely)
    to_remove.dedup();
    
    for &i in to_remove.iter().rev() {
        block.instructions.remove(i);
    }
    
    if to_remove.len() > 0 {
        println!("{to_remove:?}");
    }
    
    to_remove.len() > 0
}
