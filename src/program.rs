use std::sync::Arc;

use crate::{errors::{HRMRuntimeError, AsmParseError}, instruction::Instruction, datacube::DataCube};


pub struct Program {
    pub instructions: Arc<[Instruction]>, // TODO: remove labels altogether?
    pub initial_floor: Vec<Option<DataCube>>,
    jump_index_table: Vec<usize>,
}

impl Program {
    pub fn from_asm(asm: &str) -> Result<Self, AsmParseError> {
        let mut lines = asm.lines();
        
        if let Some(line) = lines.next() {
            if line.trim() != "-- HUMAN RESOURCE MACHINE PROGRAM --" {
                return Err(AsmParseError::MissingHeader);
            }
        } else {
            return Err(AsmParseError::EmptyFile);
        }
        
        let mut instructions = Vec::new();
        
        for line in lines {
            // strip comments
            let line = line.split("--").next().unwrap_or("");
            
            // tokenize (very basic)
            let tokens: Vec<_> = line.split_whitespace().collect();
            
            // TODO: this is a shitty hack
            if let Some(&define) = tokens.get(0) { 
                if define == "DEFINE" { break }
            }
            
            if let Some(tok) = tokens.get(2) {
                // too many tokens on a line
                return Err(AsmParseError::UnexpectedToken(tok.to_string()))
            } else if let Some(&token) = tokens.get(0) {
                // parse normally
                let instruction = Instruction::parse_from_args(token, tokens.get(1).copied())?;
                instructions.push(instruction);
            } else {
                // empty line
                continue
            }
        }
        
        let mut result = Self {
            instructions: instructions.into(),
            initial_floor: Vec::new(),
            jump_index_table: Vec::new(),
        };
        
        result.validate_jumps()?;
        
        Ok(result)
    }
    
    fn validate_jumps(&mut self) -> Result<(), AsmParseError> {
        self.jump_index_table.clear();
        
        for (i, instruction) in self.instructions.iter().enumerate() {
            match instruction {
                Instruction::_Label(x) => {
                    if x.len() != 1 {
                        // TODO: support arbitrary labels
                        return Err(AsmParseError::UnexpectedToken(x.to_string()));
                    }
                    
                    let index = x.as_bytes()[0] - 'a' as u8;
                    
                    if index as usize != self.jump_index_table.len() {
                        return Err(AsmParseError::UnexpectedToken(x.to_string()));
                    }
                    
                    self.jump_index_table.push(i);
                },
                _ => {},
            }
        }
        
        Ok(())
    }
    
    pub fn simulate(&self, mut inbox: Vec<DataCube>) -> Result<(usize, Vec<DataCube>), HRMRuntimeError> {
        inbox.reverse(); // turn the inbox into a stack
        
        let mut steps = 0;
        let mut program_counter = 0;
        let mut current_state = self.initial_floor.clone();
        let mut held_item: Option<DataCube> = None;
        let mut outbox = Vec::new();
        
        loop {
            // skip labels
            while let Some(Instruction::_Label(_)) = self.instructions.get(program_counter) {
                program_counter += 1;
            }
            
            // reached end of program
            if program_counter >= self.instructions.len() {
                break;
            }
            
            // println!("{} {program_counter}: {:?}", steps+1, self.instructions[program_counter]);
            
            match &self.instructions[program_counter] {
                // IO instructions
                Instruction::Inbox => {
                    if let Some(cube) = inbox.pop() {
                        held_item = Some(cube);
                    } else {
                        break; // reached the end of the inbox
                    }
                },
                Instruction::Outbox => {
                    if let Some(cube) = held_item.take() {
                        outbox.push(cube);
                    } else {
                        return Err(HRMRuntimeError::EmptyHands);
                    }
                },
                
                // copy instructions
                Instruction::CopyFrom(a) => {
                    let floor_tile = a.follow(&current_state)?;
                    
                    held_item = match floor_tile {
                        Some(x) => Some(x.clone()),
                        None => return Err(HRMRuntimeError::EmptyFloor),
                    };
                },
                Instruction::CopyTo(a) => {
                    let floor_tile = a.follow_mut(&mut current_state)?;
                    
                    if held_item.is_none() {
                        return Err(HRMRuntimeError::EmptyHands);
                    }
                    
                    *floor_tile = held_item.clone();
                },
                
                // arithmetic instructions
                Instruction::Add(a) => {
                    let floor_tile = a.follow(&current_state)?;
                    
                    match (held_item, floor_tile) {
                        (None, _) => return Err(HRMRuntimeError::EmptyHands),
                        (_, None) => return Err(HRMRuntimeError::EmptyFloor),
                        (_, Some(DataCube::Letter(_))) | (Some(DataCube::Letter(_)), _)
                            => return Err(HRMRuntimeError::LetterMath),
                        
                        (Some(DataCube::Number(a)), Some(DataCube::Number(b))) => {
                            held_item = Some(DataCube::from_number(a + *b)?);
                        },
                    }
                },
                Instruction::Sub(a) => {
                    let floor_tile = a.follow(&current_state)?;
                    
                    match (held_item, floor_tile) {
                        (None, _) => return Err(HRMRuntimeError::EmptyHands),
                        (_, None) => return Err(HRMRuntimeError::EmptyFloor),
                        
                        // letter vs. number subtraction is not allowed...
                        (Some(DataCube::Letter(_)), Some(DataCube::Number(_))) 
                        | (Some(DataCube::Number(_)), Some(DataCube::Letter(_)))
                            => return Err(HRMRuntimeError::LetterMath),
                        
                        // but letter vs. letter subtraction *is* allowed.
                        (Some(DataCube::Letter(a)), Some(DataCube::Letter(b))) => {
                            let result = a as i16 - *b as i16;
                            held_item = Some(DataCube::from_number(result)?);
                        },
                        
                        // (obviously, number vs. number subtraction is allowed)
                        (Some(DataCube::Number(a)), Some(DataCube::Number(b))) => {
                            held_item = Some(DataCube::from_number(a - *b)?);
                        },
                    }
                },
                Instruction::BumpUp(a) => {
                    let floor_tile = a.follow_mut(&mut current_state)?;
                    
                    match floor_tile {
                        None => return Err(HRMRuntimeError::EmptyFloor),
                        Some(DataCube::Letter(_)) => return Err(HRMRuntimeError::LetterMath),
                        Some(DataCube::Number(x)) => {
                            if *x >= 999 {
                                return Err(HRMRuntimeError::Overflow);
                            }
                            
                            *x += 1;
                        },
                    }
                    
                    held_item = floor_tile.clone();
                },
                Instruction::BumpDn(a) => {
                    let floor_tile = a.follow_mut(&mut current_state)?;
                    
                    match floor_tile {
                        None => return Err(HRMRuntimeError::EmptyFloor),
                        Some(DataCube::Letter(_)) => return Err(HRMRuntimeError::LetterMath),
                        Some(DataCube::Number(x)) => {
                            if *x <= -999 {
                                return Err(HRMRuntimeError::Overflow);
                            }
                            
                            *x -= 1;
                        },
                    }
                    
                    held_item = floor_tile.clone();
                },
                
                // jump instructions
                Instruction::Jump(label) => {
                    if label.len() != 1 {
                        // TODO: support arbitrary labels
                        panic!("Unknown label: {}", label);
                    }
                    
                    let index = label.as_bytes()[0] - 'a' as u8;
                    
                    program_counter = self.jump_index_table[index as usize];
                },
                Instruction::JumpN(label) => {
                    if label.len() != 1 {
                        // TODO: support arbitrary labels
                        panic!("Unknown label: {}", label);
                    }
                    
                    let index = label.as_bytes()[0] - 'a' as u8;
                    
                    match held_item {
                        Some(DataCube::Number(x)) if x < 0 => {
                            program_counter = self.jump_index_table[index as usize];
                        },
                        Some(_) => {},
                        None => return Err(HRMRuntimeError::EmptyHands),
                    }
                },
                Instruction::JumpZ(label) => {
                    if label.len() != 1 {
                        // TODO: support arbitrary labels
                        panic!("Unknown label: {}", label);
                    }
                    
                    let index = label.as_bytes()[0] - 'a' as u8;
                    
                    match held_item {
                        Some(DataCube::Number(x)) if x == 0 => {
                            program_counter = self.jump_index_table[index as usize];
                        },
                        Some(_) => {},
                        None => return Err(HRMRuntimeError::EmptyHands),
                    }
                },
                
                Instruction::_Label(_) => { unreachable!() },
            }
            
            steps += 1;
            program_counter += 1;
        }
        
        Ok((steps, outbox))
    }
    
    pub fn split_into_blocks(&self) -> Vec<crate::basic_blocks::BasicBlock> {
        use crate::basic_blocks::{BasicBlockId, BasicBlock, JumpFlag};
        use Instruction::*;
        
        // find leaders, s.t. each pair in leader_indices is the start and end of a block
        let mut leader_indices = vec![0];
        for (i, inst) in self.instructions.iter().enumerate().skip(1) {
            if let Jump(_) | JumpN(_) | JumpZ(_) = inst {
                if !matches!(self.instructions.get(i + 1), Some(Jump(_) | JumpN(_) | JumpZ(_))) {
                    leader_indices.push(i + 1);
                }
            }
            else if let _Label(_) = inst {
                if let Some(_Label(_)) = self.instructions.get(i - 1) {
                    continue;
                }
                // dont add the same index twice
                if let Some(&a) = leader_indices.last() {
                    if a == i { continue; }
                }
                leader_indices.push(i);
            }
        }
        
        // make sure to end the last block
        if let Some(&last) = leader_indices.last() {
            if last != self.instructions.len() {
                leader_indices.push(self.instructions.len());
            }
        }
        
        // ids of each block that a given label is the head for
        let label_block_ids: Vec<BasicBlockId> = self.jump_index_table.iter()
            .map(|i| match leader_indices.binary_search(i) {
                Ok(idx) => BasicBlockId(idx),
                Err(idx) => BasicBlockId(idx - 1),
            }).collect();
        
        let mut blocks: Vec<_> = leader_indices.iter()
        .zip(leader_indices.iter().skip(1)).enumerate()
        .map(|(i, (&(mut a), &(mut b)))| {
            let end = b;
            
            // advance a forward to ignore labels
            while let Some(_Label(_)) = self.instructions.get(a) {
                a += 1;
            }
            
            // advance b backwards to ignore jumps
            while let Some(Jump(_) | JumpN(_) | JumpZ(_)) = self.instructions.get(b-1) {
                b -= 1;
            }
            
            let mut has_unconditional_jump = false;
            
            let mut jumps: Vec<(BasicBlockId, JumpFlag)> = self.instructions[b..end].iter().map(|jump| {
                let (label, flag) = match jump {
                    Jump(l) => {
                        has_unconditional_jump = true;
                        (l, JumpFlag::Always)
                    },
                    JumpN(l) => (l, JumpFlag::IfNegative),
                    JumpZ(l) => (l, JumpFlag::IfZero),
                    _ => unreachable!(),
                };
                let label_idx = label.as_bytes()[0] - b'a';
                let id = label_block_ids[label_idx as usize].clone();
                (id, flag)
            }).collect();
            
            if !has_unconditional_jump {
                jumps.push((BasicBlockId(i + 1), JumpFlag::Always));
            }
            
            BasicBlock {
                id: BasicBlockId(i),
                instructions: self.instructions[a..b].to_vec(),
                outgoing_jumps: jumps,
                incoming_jumps: vec![],
            }
        }).collect();
        
        // add incoming jumps
        let n = blocks.len();
        for i in 0..blocks.len() {
            let (_blocks, blocks_after) = blocks.split_at_mut(i+1);
            let (block, blocks_before) = _blocks.split_last_mut().unwrap();
            
            for &(BasicBlockId(id), flag) in &block.outgoing_jumps {
                if id == block.id.0 || id == n {
                    continue;
                }
                
                let target_block = if id < block.id.0 {
                    &mut blocks_before[id]
                } else {
                    &mut blocks_after[id - block.id.0 - 1]
                };
                
                let incoming_flag = if block.outgoing_jumps.len() == 1 {
                    Some(flag)
                } else {
                    None // TODO
                };
                
                target_block.incoming_jumps.push((block.id.clone(), incoming_flag));
            }
        }
        
        blocks
    }
}