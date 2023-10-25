use crate::optimize::jump_flag::JumpFlag;

use super::control_flow_graph::ProgramControlFlowGraph;

pub fn remove_dead_blocks(graph: &mut ProgramControlFlowGraph) -> bool {
    let old_len = graph.blocks.len();
    graph.blocks.retain(|block| block.id.0 == 0 || !block.incoming_jumps.is_empty());
    old_len != graph.blocks.len()
}

pub fn combine_sequential_blocks(graph: &mut ProgramControlFlowGraph) -> bool {
    let mut i = 0;
    let mut offset = 0;
    
    let mut to_remove: Vec<usize> = vec![];
    
    while i + offset < graph.blocks.len() - 1 {
        let (_blocks, _blocks_after) = graph.blocks.split_at_mut(i+1);
        let block1 = _blocks.last_mut().unwrap();
        let block2 = _blocks_after.get_mut(offset).unwrap();
        
        match (&block1.outgoing_jumps[..], &block2.incoming_jumps[..]) {
            ([(b, JumpFlag::Always)], [(_, JumpFlag::Always)]) if /* *a == block1.id && */ *b == block2.id => {
                block1.instructions.extend(block2.instructions.drain(..));
                block1.outgoing_jumps = block2.outgoing_jumps.clone();
                to_remove.push(i+offset+1);
                offset += 1;
            }
            _ => { i = i + offset + 1; offset = 0; continue }
        }
    }
    
    for &i in to_remove.iter().rev() {
        graph.blocks.remove(i);
    }
    
    !to_remove.is_empty()
}

/// NOTE: soundness depends on `simplify_outgoing_jumps` and 
///       then `refresh_incoming_jumps` being run before this.
pub fn remove_empty_blocks(graph: &mut ProgramControlFlowGraph) -> bool {
    let mut to_remove: Vec<usize> = vec![];
    
    for (i, block) in graph.blocks.iter().enumerate() {
        if block.instructions.is_empty() {
            to_remove.push(i);
        }
    }
    
    for &i in to_remove.iter().rev() {
        // can't just remove the block, because it might have incoming jumps
        let current_block_id = graph.blocks[i].id.clone();
        let incoming_jumps = graph.blocks[i].incoming_jumps.clone();
        let outgoing_jumps = graph.blocks[i].outgoing_jumps.clone();
        
        for (id, flag) in incoming_jumps {
            let block_idx = graph.blocks.iter().position(|block| block.id == id).expect("invalid block id");
            let block = &mut graph.blocks[block_idx];
            
            // replace the incoming jump with all the jumps from the block we're removing
            // NOTE: since we know that all the jumps in the preceeding block commute (we
            //       just ran `simplify_outgoing_jumps`), we can just append them all to
            //       the end of its outgoing jumps.
            let jump_pos = block.outgoing_jumps.iter()
                .position(|(id2, _)| *id2 == current_block_id)
                .expect("incoming jump not found");
            
            let (_block_id, _out_flag) = block.outgoing_jumps.remove(jump_pos);
            debug_assert_eq!(_block_id, current_block_id);
            debug_assert_eq!(flag, _out_flag);
            
            block.outgoing_jumps.extend(outgoing_jumps.iter().map(|(id_out, flag_out)| (id_out.clone(), *flag_out & flag)));
        }
        
        graph.blocks.remove(i);
    }
    
    !to_remove.is_empty()
}
