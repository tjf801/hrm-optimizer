use std::{sync::Arc, collections::HashMap};

use crate::{errors::{HRMRuntimeError, AsmParseError}, instruction::Instruction, datacube::DataCube};


pub struct Program {
    pub instructions: Arc<[Instruction]>,
    pub initial_floor: Vec<Option<DataCube>>,
    pub jump_label_lines: std::collections::HashMap<String, usize>,
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
        
        let mut label_lines = HashMap::<String, usize>::new();
        
        let mut instructions = Vec::new();
        
        for line in lines {
            // strip comments
            let line = line.split("--").next().unwrap_or("");
            
            // tokenize (very basic)
            let tokens: Vec<_> = line.split_whitespace().collect();
            
            // TODO: this is a shitty hack
            if let Some(&define) = tokens.get(0) { 
                if define == "DEFINE" { break }
                if define == "COMMENT" { continue }
            }
            
            if let Some(tok) = tokens.get(2) {
                // too many tokens on a line
                return Err(AsmParseError::UnexpectedToken(tok.to_string()))
            } else if let Some(&token) = tokens.get(0) {
                // parse labels
                if let Some(label) = token.strip_suffix(':') {
                    if let Some(arg) = tokens.get(1) {
                        return Err(AsmParseError::UnexpectedToken(arg.to_string()))
                    }
                    label_lines.insert(label.to_string(), instructions.len());
                } else {
                    // parse normally
                    let instruction = Instruction::parse_from_args(token, tokens.get(1).copied())?;
                    instructions.push(instruction);
                }
            } else {
                // empty line
                continue
            }
        }
        
        // make sure all jumps work
        Self::validate_jumps(&instructions, &label_lines)?;
        
        Ok(Self {
            instructions: instructions.into(),
            initial_floor: Vec::new(),
            jump_label_lines: label_lines,
        })
    }
    
    fn validate_jumps(instructions: &[Instruction], labels: &HashMap<String, usize>) -> Result<(), AsmParseError> {
        for instr in instructions {
            match instr {
                Instruction::Jump(label) | Instruction::JumpN(label) | Instruction::JumpZ(label) => {
                    if !labels.contains_key(label) {
                        return Err(AsmParseError::UnknownLabel(label.clone()));
                    }
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
                    program_counter = self.jump_label_lines[label].saturating_sub(1);
                },
                Instruction::JumpN(label) => {
                    match held_item {
                        Some(DataCube::Number(x)) if x < 0 => {
                            program_counter = self.jump_label_lines[label].saturating_sub(1);
                        },
                        Some(_) => {},
                        None => return Err(HRMRuntimeError::EmptyHands),
                    }
                },
                Instruction::JumpZ(label) => {
                    match held_item {
                        Some(DataCube::Number(x)) if x == 0 => {
                            program_counter = self.jump_label_lines[label].saturating_sub(1);
                        },
                        Some(_) => {},
                        None => return Err(HRMRuntimeError::EmptyHands),
                    }
                },
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
        }
        
        for jump_idx in self.jump_label_lines.values() {
            if !leader_indices.contains(jump_idx) {
                leader_indices.push(*jump_idx);
            }
        }
        
        leader_indices.sort();
        
        // make sure to end the last block
        if let Some(&last) = leader_indices.last() {
            if last != self.instructions.len() {
                leader_indices.push(self.instructions.len());
            }
        }
        
        leader_indices.iter()
        .zip(leader_indices.iter().skip(1)).enumerate()
        .map(|(i, (&a, &(mut b)))| {
            let end = b;
            
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
                let label_idx = self.jump_label_lines[label];
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
                instructions: self.instructions[a..b].to_vec(),
                outgoing_jumps: jumps,
                incoming_jumps: vec![],
            }
        }).collect()
    }
}