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

    #[error("Invalid quantity: {0} (must be 1-1000)")]
    InvalidQuantity(i32),

    #[error("Invalid die size: d{0}")]
    InvalidDieSize(i32),

    #[error("Quantity limit exceeded: {0} (maximum is 1000)")]
    QuantityLimitExceeded(i32),

    #[error("{0} cannot be a float: {1}")]
    FloatParseError(DiceComponent, f64),

    #[error("Parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}
