pub mod condition;

pub mod dispatch;

pub mod execute;

#[allow(unused)] // while we wait for the debug interface
pub mod meta;

// we're suppressing this lint to keep the naming consistent with the pan docs
#[allow(clippy::upper_case_acronyms)]
pub mod operand;

// we're suppressing this lint to keep the naming consistent with the pan docs
#[allow(clippy::upper_case_acronyms)]
pub mod parameter;

// pub mod opcode;

#[derive(Debug)]
pub struct OpcodeExtractionError; // NOTE: not convinced if this should stay here or move to somewhere else
