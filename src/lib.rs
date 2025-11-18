mod error;
mod model;

pub use error::DiceError;
pub use model::RollResult;

pub fn roll(expression: &str) -> Result<RollResult, DiceError> {
    Err(DiceError::InvalidFormat("Not implemented".to_string()))
}

pub fn roll_with_seed(expression: &str, seed: [u8; 32]) -> Result<RollResult, DiceError> {
    // We'll implement this later
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
