use stylus_sdk::prelude::*;

#[derive(SolidityError, Debug)]
pub enum AfroCreateError {
    #[solidity(string)]
    Unauthorized(String),
    
    #[solidity(string)]
    InvalidInput(String),
    
    #[solidity(string)]
    InsufficientFunds(String),
    
    #[solidity(string)]
    ProjectNotFound(String),
    
    #[solidity(string)]
    InvalidENSName(String),
    
    #[solidity(string)]
    ValidationFailed(String),
    
    #[solidity(string)]
    FundingDeadlinePassed(String),
    
    #[solidity(string)]
    AlreadyExists(String),
    
    #[solidity(string)]
    NotActive(String),
    
    #[solidity(string)]
    InsufficientValidators(String),
    
    #[solidity(string)]
    ReentrancyGuard(String),
    
    #[solidity(string)]
    ContractPaused(String),
    
    #[solidity(string)]
    InvalidAddress(String),
    
    #[solidity(string)]
    TransferFailed(String),
    
    #[solidity(string)]
    OracleError(String),
}

pub type Result<T> = core::result::Result<T, AfroCreateError>;

pub fn require_authorized(condition: bool, message: &str) -> Result<()> {
    if !condition {
        Err(AfroCreateError::Unauthorized(message.to_string()))
    } else {
        Ok(())
    }
}

pub fn require_valid_input(condition: bool, message: &str) -> Result<()> {
    if !condition {
        Err(AfroCreateError::InvalidInput(message.to_string()))
    } else {
        Ok(())
    }
}

pub fn require_sufficient_funds(condition: bool, message: &str) -> Result<()> {
    if !condition {
        Err(AfroCreateError::InsufficientFunds(message.to_string()))
    } else {
        Ok(())
    }
}

pub fn require_active(condition: bool, message: &str) -> Result<()> {
    if !condition {
        Err(AfroCreateError::NotActive(message.to_string()))
    } else {
        Ok(())
    }
}