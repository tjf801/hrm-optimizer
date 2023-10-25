use crate::{
    program::Program,
    optimize::basic_blocks::{BasicBlockId, BasicBlock, JumpFlag},
    instruction::Instruction,
    datacube::DataCube
};

pub struct ProgramControlFlowGraph {
    pub initial_floor: Vec<Option<DataCube>>,
    pub blocks: Vec<BasicBlock>,
}

impl ProgramControlFlowGraph {
    /// creates a control flow graph from a program.
    pub fn new(program: &Program) -> Self {
        use Instruction::{Jump, JumpN, JumpZ};
        
        // find leaders, s.t. each pair in leader_indices is the start and end of a block
        let mut leader_indices = vec![0];
        
        for (i, inst) in program.instructions.iter().enumerate().skip(1) {
            if let Jump(_) | JumpN(_) | JumpZ(_) = inst {
                if !matches!(program.instructions.get(i + 1), Some(Jump(_) | JumpN(_) | JumpZ(_))) {
                    leader_indices.push(i + 1);
                }
            }
        }
        
        for jump_idx in program.jump_label_lines.values() {
            if !leader_indices.contains(jump_idx) {
                leader_indices.push(*jump_idx);
            }
        }
        
        leader_indices.sort();
        
        // make sure to end the last block
        if let Some(&last) = leader_indices.last() {
            if last != program.instructions.len() {
                leader_indices.push(program.instructions.len());
            }
        }
        
        let blocks = leader_indices.iter()
        .zip(leader_indices.iter().skip(1)).enumerate()
        .map(|(i, (&a, &(mut b)))| {
            let end = b;
            
            // advance b backwards to ignore jumps
            while let Some(Jump(_) | JumpN(_) | JumpZ(_)) = program.instructions.get(b-1) {
                b -= 1;
            }
            
            let mut has_unconditional_jump = false;
            
            let mut jumps: Vec<(BasicBlockId, JumpFlag)> = program.instructions[b..end].iter().map(|jump| {
                let (label, flag) = match jump {
                    Jump(l) => {
                        has_unconditional_jump = true;
                        (l, JumpFlag::Always)
                    },
                    JumpN(l) => (l, JumpFlag::IfNegative),
                    JumpZ(l) => (l, JumpFlag::IfZero),
                    _ => unreachable!(),
                };
                let label_idx = program.jump_label_lines[label];
                let block_id = match leader_indices.binary_search(&label_idx) {
                    Ok(idx) => BasicBlockId(idx),
                    Err(idx) => BasicBlockId(idx - 1),
                };
                (block_id, flag)
            }).collect();
            
            if !has_unconditional_jump {
                jumps.push((BasicBlockId(i + 1), JumpFlag::Always));
            }
            
            BasicBlock {
                id: BasicBlockId(i),
                instructions: program.instructions[a..b].to_vec(),
                outgoing_jumps: jumps,
                incoming_jumps: vec![],
            }
        }).collect();
        
        let mut result = Self {
            initial_floor: program.initial_floor.clone(),
            blocks,
        };
        
        result.refresh_incoming_jumps();
        
        result
    }
    
    /// update all incoming jumps for each block.
    /// 
    /// it is MANDATORY to call this function after modifying the outgoing jumps of any block.
    fn refresh_incoming_jumps(&mut self) {
        let mut block_ids = Vec::new();
        for block in self.blocks.iter_mut() {
            block.incoming_jumps.clear();
            block_ids.push(block.id.clone());
        }
        let block_ids = block_ids;
        
        // redo incoming jumps
        for i in 0..self.blocks.len() {
            let (_blocks, blocks_after) = self.blocks.split_at_mut(i+1);
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
}
