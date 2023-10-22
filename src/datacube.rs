use crate::errors::HRMRuntimeError;

#[derive(PartialEq, Eq, Clone)]
pub enum DataCube {
    /// value between -999 and 999
    Number(i16),
    
    /// letter between A and Z
    Letter(u8),
}

impl DataCube {
    pub fn from_number<T: TryInto<i16>>(n: T) -> Result<Self, HRMRuntimeError> {
        let n = TryInto::<i16>::try_into(n).map_err(|_| HRMRuntimeError::Overflow)?;
        
        if n < -999 || n > 999 {
            return Err(HRMRuntimeError::Overflow)
        }
        
        Ok(Self::Number(n))
    }
    
    pub fn from_char(c: char) -> Option<Self> {
        let c: u8 = c.try_into().ok()?;
        
        if c.is_ascii_uppercase() {
            Some(Self::Letter(c))
        } else {
            None
        }
    }
}

impl std::fmt::Debug for DataCube {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Datacube({})", match self {
            Self::Number(x) => x.to_string(),
            Self::Letter(x) => format!("'{}'", *x as char),
        })
    }
}

impl std::fmt::Display for DataCube {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { todo!() }
}
