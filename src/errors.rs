use crate::datacube::DataCube;

/// When optimizing a program, it is advantageous to treat
/// these as "undefined behavior" and assume they never happen.
#[derive(Debug)]
pub enum HRMRuntimeError {
    /// Empty value! You can't {operation} with an empty tile on the floor! Try writing something to that tile first.
    /// 
    /// operation: COPYFROM, ADD, SUB, BUMPUP, BUMPDN
    EmptyFloor,
    
    /// Empty value! You can't {operation} with empty hands!
    /// 
    /// operation: OUTBOX, COPYTO, JUMPZ, JUMPN, ADD, SUB, 
    EmptyHands,
    
    /// You can't {operation} a letter! What would that even mean?!
    /// 
    /// operation: ADD, SUB (when 2nd operand is a number), BUMPUP, BUMPDN
    LetterMath,
    
    /// Bad tile address! Tile with address {addr: u32} does not exist! Where do you think you're going?
    BadTileAddress,
    
    /// Bad tile address! You can't indirect to a tile with a letter like "{letter}". Only numbers allowed! Where do you think you're going?
    LetterAddress,
    
    /// Overflow! Each data unit is restricted to values between -999 and 999. That should be enough for anybody.
    Overflow,
}

impl std::fmt::Display for HRMRuntimeError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        todo!()
    }
}

impl std::error::Error for HRMRuntimeError {}


/// Errors that can occur when running tests
#[allow(dead_code)]
#[derive(Debug)]
pub enum HRMTestError {
    /// Not enough stuff in the OUTBOX! Management expected a total of {expected: usize} items, not {actual: usize}!
    NotEnoughOutBox{ actual: usize, expected: usize },
    
    /// Bad outbox! Management expected {test: Datacube}, but you outboxed {actual: Datacube}.
    BadOutbox{ actual: DataCube, expected: DataCube },
    
    /// Aha! Your solution works with those
    /// specific inputs... but it FAILS on other
    /// possible inputs! Yes, here, I'll give you
    /// some inputs that cause your solution
    /// to fail, so you can see for yourself.
    SolutionNotRobust,
}

impl std::fmt::Display for HRMTestError {
    fn fmt(&self, fmtr: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Self::NotEnoughOutBox { actual, expected }
            => fmtr.write_fmt(format_args!("Not enough stuff in the OUTBOX! Management expected a total of {expected} items, not {actual}!")),
            Self::BadOutbox { actual, expected }
            => fmtr.write_fmt(format_args!("Bad outbox! Management expected {expected}, but you outboxed {actual}.")),
            Self::SolutionNotRobust
            => fmtr.write_str("\"Aha! Your solution works with those specific inputs... but it FAILS on other possible inputs! Yes, here, I'll give you some inputs that cause your solution to fail, so you can see for yourself.\""),
        }
    }
}

impl std::error::Error for HRMTestError {}


#[derive(Debug)]
pub enum AsmParseError {
    EmptyFile,
    MissingHeader,
    UnexpectedToken(String),
    ExpectedToken,
    IntParseError(std::num::ParseIntError),
    UnknownLabel(String)
}
