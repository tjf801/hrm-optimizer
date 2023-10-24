use crate::basic_blocks::{BasicBlock, JumpFlag};

pub fn refresh_incoming_jumps(blocks: &mut [BasicBlock]) {
    let mut block_ids = Vec::new();
    for block in blocks.iter_mut() {
        block.incoming_jumps.clear();
        block_ids.push(block.id.clone());
    }
    let block_ids = block_ids;
    
    // redo incoming jumps
    for i in 0..blocks.len() {
        let (_blocks, blocks_after) = blocks.split_at_mut(i+1);
        let (block, blocks_before) = _blocks.split_last_mut().unwrap();
        
        for (out_jmp_id, flag) in &block.outgoing_jumps {
            if out_jmp_id == &block.id {
                continue;
            }
            
            let jmp_idx = match block_ids.binary_search(out_jmp_id) {
                Ok(i) => i,
                Err(_) => continue,
            };
            
            let target_block = if jmp_idx < i {
                &mut blocks_before[jmp_idx]
            } else {
                &mut blocks_after[jmp_idx - i - 1]
            };
            
            let incoming_flag = flag;
            
            target_block.incoming_jumps.push((block.id.clone(), *incoming_flag));
        }
    }
}

pub fn simplify_outgoing_jumps(blocks: &mut [BasicBlock]) {
    for block in blocks.iter_mut() {
        match &mut block.outgoing_jumps[..] {
            [(_, flag1), (_, flag2)] => {
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
            }
            _ => {}
        }
    }
    
    refresh_incoming_jumps(blocks);
}

pub fn remove_dead_blocks(blocks: &mut Vec<BasicBlock>) {
    blocks.retain(|block| block.id.0 == 0 || !block.incoming_jumps.is_empty());
    refresh_incoming_jumps(blocks);
}

pub fn combine_sequential_blocks(blocks: &mut Vec<BasicBlock>) {
    let mut i = 0;
    let mut offset = 0;
    
    while i + offset < blocks.len() - 1 {
        let (_blocks, _blocks_after) = blocks.split_at_mut(i+1);
        let block1 = _blocks.last_mut().unwrap();
        let block2 = _blocks_after.get_mut(offset).unwrap();
        
        match (&block1.outgoing_jumps[..], &block2.incoming_jumps[..]) {
            ([(b, JumpFlag::Always)], [(a, JumpFlag::Always)]) if *a == block1.id && *b == block2.id => {
                block1.instructions.extend(block2.instructions.drain(..));
                block1.outgoing_jumps = block2.outgoing_jumps.clone();
                offset += 1;
            }
            _ => { i = i + offset + 1; offset = 0; continue }
        }
    }
    
    refresh_incoming_jumps(blocks);
    remove_dead_blocks(blocks);
}
