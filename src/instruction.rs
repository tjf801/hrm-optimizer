use crate::{errors::{AsmParseError, HRMRuntimeError}, datacube::DataCube};


#[derive(Debug)]
pub enum Address {
    Direct(usize),
    Indirect(usize),
}

impl Address {
    fn parse(s: &str) -> Result<Self, AsmParseError> {
        // either [integer] or integer
        if let Some(s) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            Ok(Address::Indirect(s.parse().map_err(|x| AsmParseError::IntParseError(x))?))
        } else {
            Ok(Address::Direct(s.parse().map_err(|x| AsmParseError::IntParseError(x))?))
        }
    }
    
    fn raw_address(&self, floor: &Vec<Option<DataCube>>) -> Result<usize, HRMRuntimeError> {
        match self {
            Address::Direct(x) => Ok(*x),
            Address::Indirect(x) => {
                if let Some(tile) = floor.get(*x as usize) {
                    match tile {
                        Some(DataCube::Number(x)) => {
                            TryInto::<usize>::try_into(*x).map_err(|_| HRMRuntimeError::BadTileAddress)
                        },
                        Some(DataCube::Letter(_)) => Err(HRMRuntimeError::LetterAddress),
                        None => Err(HRMRuntimeError::EmptyFloor), // TODO: double check
                    }
                } else {
                    Err(HRMRuntimeError::BadTileAddress)
                }
            },
        }
    }
    
    pub fn follow<'a>(&self, floor: &'a Vec<Option<DataCube>>) -> Result<&'a Option<DataCube>, HRMRuntimeError> {
        let i = self.raw_address(floor)?;
        
        match floor.get(i) {
            Some(x) => Ok(x),
            None => Err(HRMRuntimeError::BadTileAddress),
        }
    }
    
    pub fn follow_mut<'a>(&self, floor: &'a mut Vec<Option<DataCube>>) -> Result<&'a mut Option<DataCube>, HRMRuntimeError> {
        let i = self.raw_address(floor)?;
        
        match floor.get_mut(i) {
            Some(x) => Ok(x),
            None => Err(HRMRuntimeError::BadTileAddress),
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    /// #### INBOX: Pick up the next thing from the inbox.
    /// 
    /// Pops the top value from the INBOX and copies it into the accumulator.
    /// 
    /// If the INBOX is empty, this instruction ends the program.
    Inbox,
    
    /// #### OUTBOX: Put whatever you are holding into the OUTBOX.
    /// 
    /// Pushes the accumulator into the OUTBOX, and clears the accumulator.
    /// 
    /// Triggers an error if the accumulator is empty.
    Outbox,
    
    /// #### COPYFROM: Walk to a specific tile on the floor and pick up a copy of whatever is there.
    /// 
    /// Copies the value from the given address into the accumulator.
    /// 
    /// Triggers an error if the address is invalid or empty.
    CopyFrom(Address),
    
    /// #### COPYTO: Copy whatever you are currently holding to a specific tile on the floor.
    /// 
    /// copy the value from the accumulator to the address
    CopyTo(Address),
    
    /// #### ADD: Add the contents of a specific tile on the floor to whatever you are currently holding. The result goes back into your hands.
    /// 
    /// add the value at the address to the accumulator
    Add(Address),
    
    /// #### SUB: Subtract the contents of a specific tile on the floor FROM whatever you are currently holding. The result goes back into your hands.
    /// 
    /// subtract the value at the address from the accumulator
    Sub(Address),
    
    /// #### BUMP+: Add ONE to the contents of a specific tile on the floor. The result is written back to the floor, and also back into your hands for your convenience!
    /// 
    /// increment the value at the address, and copy it to the accumulator
    BumpUp(Address),
    
    /// **BUMP-: Subtract ONE from the contents of a specific tile on the floor.**
    /// **The result is written back to the floor, and also back into your hands for your convenience!**
    /// 
    /// decrement the value at the address, and copy it to the accumulator
    BumpDn(Address),
    
    /// internal helper instruction to show the end point of a jump instruction
    _Label(String),
    
    /// **JUMP: Jump to a new location within your program.**
    /// **You can jump backward to create loops, or jump forward to skip entire sections.**
    /// **The possibilities are endless!**
    /// 
    /// unconditionally jump to the label
    Jump(String),
    
    /// **JUMP IF ZERO: Jump only if you are currently holding a ZERO.**
    /// **Otherwise continue to the next line in your program.**
    /// 
    /// jump to the label if the accumulator is zero
    JumpZ(String),
    
    /// **JUMP IF NEGATIVE: Jump only if you are currently holding a negative number.**
    /// **Otherwise continue to the next line in your program.**
    /// 
    /// jump to the label if the accumulator is negative
    JumpN(String),
}

impl Instruction {
    pub fn parse_from_args(statement: &str, arg: Option<&str>) -> Result<Self, AsmParseError> {
        // parse labels
        if let Some(label) = statement.strip_suffix(':') {
            return if let Some(arg) = arg {
                Err(AsmParseError::UnexpectedToken(arg.to_string()))
            } else {
                Ok(Self::_Label(label.to_string()))
            }
        }
        
        // parse everything else
        match statement {
            "INBOX" => {
                arg
                    .map(|s| Err(AsmParseError::UnexpectedToken(s.to_string())))
                    .unwrap_or(Ok(Self::Inbox))
            },
            "OUTBOX" => {
                arg
                    .map(|s| Err(AsmParseError::UnexpectedToken(s.to_string())))
                    .unwrap_or(Ok(Self::Outbox))
            },
            
            "COPYFROM" => Ok(Self::CopyFrom(Address::parse(arg.ok_or(AsmParseError::ExpectedToken)?)?)),
            "COPYTO" => Ok(Self::CopyTo(Address::parse(arg.ok_or(AsmParseError::ExpectedToken)?)?)),
            "ADD" => Ok(Self::Add(Address::parse(arg.ok_or(AsmParseError::ExpectedToken)?)?)),
            "SUB" => Ok(Self::Sub(Address::parse(arg.ok_or(AsmParseError::ExpectedToken)?)?)),
            "BUMPUP" => Ok(Self::BumpUp(Address::parse(arg.ok_or(AsmParseError::ExpectedToken)?)?)),
            "BUMPDN" => Ok(Self::BumpDn(Address::parse(arg.ok_or(AsmParseError::ExpectedToken)?)?)),
            
            "JUMP" => Ok(Self::Jump(arg.ok_or(AsmParseError::ExpectedToken)?.to_string())),
            "JUMPZ" => Ok(Self::JumpZ(arg.ok_or(AsmParseError::ExpectedToken)?.to_string())),
            "JUMPN" => Ok(Self::JumpN(arg.ok_or(AsmParseError::ExpectedToken)?.to_string())),
            
            tok => Err(AsmParseError::UnexpectedToken(tok.to_string())),
        }
    }
}
