use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiceError {
    #[error("Invalid dice notation format: {0}")]
    InvalidFormat(String),

    #[error("Invalid quantity: {0} (must be 1-1000)")]
    InvalidQuantity(i32),

    #[error("Invalid die size: d{0} (must be positive)")]
    InvalidDieSize(i32),

    #[error("Quantity limit exceeded: {0} (maximum is 1000)")]
    QuantityLimitExceeded(i32),

    #[error("Parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}
