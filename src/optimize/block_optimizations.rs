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
