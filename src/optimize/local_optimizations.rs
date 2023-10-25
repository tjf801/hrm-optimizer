use crate::{optimize::basic_blocks::BasicBlock, instruction::Address};

use super::basic_blocks::BasicBlockId;
use super::jump_flag::JumpFlag;
use super::control_flow_graph::{ProgramControlFlowGraph, Optimization};

/// convert a function that optimizes a single block into an optimization pass
/// for a full control flow graph.
pub fn local_optimization<T: FnMut(&mut BasicBlock) -> bool>(mut block_optimization: T) -> impl Optimization {
    move |graph: &mut ProgramControlFlowGraph| {
        let mut modified = false;
        for block in graph.blocks.iter_mut() {
            modified |= block_optimization(block);
        }
        modified
    }
}

/// make all outgoing jumps in a given block as simple as possible.
pub fn simplify_outgoing_jumps(block: &mut BasicBlock) -> bool {
    let mut result = false;
    
    // transforms `JUMPIF(cond1) a; JUMPIF(cond2) b;`
    // into `JUMPIF(cond1) a; JUMPIF(cond1 && !cond2) b;`
    // NOTE: after this transformation, *all* jumps in the block are able to be arbitrarily shuffled around
    for i in 0..block.outgoing_jumps.len() {
        for j in i+1..block.outgoing_jumps.len() {
            let flag = block.outgoing_jumps[i].1;
            let old_flag = block.outgoing_jumps[j].1;
            block.outgoing_jumps[j].1 &= !flag;
            result |= block.outgoing_jumps[j].1 != old_flag;
        }
    }
    
    // combine all jumps to the same block
    // e.g. `JUMPIF(cond1) a; JUMPIF(cond2) a;` becomes `JUMPIF(cond1 || cond2) a;`
    let mut uniq = std::collections::HashMap::new();
    for (BasicBlockId(target), cond) in block.outgoing_jumps.iter() {
        match uniq.get_mut(target) {
            Some(existing_cond) => {
                *existing_cond |= *cond;
                result = true;
            },
            None => {
                uniq.insert(*target, *cond);
            }
        }
    }
    
    // remove all jumps with a condition of `Never` and re-assign to jumps
    block.outgoing_jumps = uniq.iter().filter_map(|(&id, &cond)| {
        if cond == JumpFlag::Never {
            result = true;
            None
        } else {
            Some((BasicBlockId(id), cond))
        }
    }).collect();
    
    // the block needs to always jump *somewhere*, so this is just a sanity check
    debug_assert_eq!(block.outgoing_jumps.iter().map(|(_, c)| *c).reduce(|a, b| a | b).unwrap(), JumpFlag::Always);
    
    result
}

/// perform peephole optimizations in a given block.
/// 
/// it should be noted that this doesn't involve any real dataflow analysis,
/// dependency analysis, or anything like that. it's just a bunch of simple
/// optimizations that are easy to implement and are only really likely to
/// happen after multiple blocks are merged into one.
pub fn peephole_optimizations(block: &mut BasicBlock) -> bool {
    use crate::instruction::Instruction::*;
    
    let mut to_remove = Vec::new();
    
    // length two optimizations
    for (i, instrs) in block.instructions.windows(2).enumerate() {
        match instrs {
            [ // statically detectable undefined behavior
                Outbox,
                Outbox | CopyTo(_) | Add(_) | Sub(_), // TODO: jumpz and jumpn?
            ] => {
                eprintln!("Undefined behavior warning! An OUTBOX followed by a {instr:?} instruction will always raise a EmptyHands error.", instr = instrs[1]);
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
    
    to_remove.len() > 0
}
