use std::fmt;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DiceComponent {
    Quantity,
    Sides,
    Modifier,
}

impl fmt::Display for DiceComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiceComponent::Quantity => write!(f, "Quantity"),
            DiceComponent::Sides => write!(f, "Die size"),
            DiceComponent::Modifier => write!(f, "Modifier"),
        }
    }
}

#[derive(Error, Debug)]
pub enum DiceError {
    #[error("Invalid dice notation format: {0}")]
    InvalidFormat(String),

    #[error("{0} is too low: {1}")]
    BelowMinimum(DiceComponent, i32),

    #[error("{0} limit exceeded: {1}")]
    LimitExceeded(DiceComponent, i32),

    #[error("{0} cannot be a float: {1}")]
    FloatParseError(DiceComponent, f64),

    #[error("Parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}
