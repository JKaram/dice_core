use std::fmt;
use thiserror::Error;

/// Identifies which component of a dice expression caused an error.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DiceComponent {
    /// The number of dice to roll.
    Quantity,
    /// The number of sides on each die.
    Sides,
    /// The numeric modifier added to/subtracted from the total.
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

/// Errors that can occur when parsing or rolling dice.
#[derive(Error, Debug)]
pub enum DiceError {
    /// The expression syntax is invalid or contains trailing characters.
    #[error("Invalid dice notation format: {0}")]
    InvalidFormat(String),

    /// A value is below the minimum (must be >= 1).
    #[error("{0} is too low: {1}")]
    BelowMinimum(DiceComponent, i32),

    /// A value exceeds the maximum limit.
    #[error("{0} limit exceeded: {1}")]
    LimitExceeded(DiceComponent, i32),

    /// A floating-point number was used where an integer is required.
    #[error("{0} cannot be a float: {1}")]
    FloatParseError(DiceComponent, f64),

    /// An integer parsing error occurred.
    #[error("Parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}
