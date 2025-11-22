mod error;
mod model;
mod parser;

pub use error::DiceError;
pub use model::RollResult;
pub use parser::{DiceRequest, dice_result};

pub fn roll(expression: &str) -> Result<RollResult, DiceError> {
    Err(DiceError::InvalidFormat("Not implemented".to_string()))
}

pub fn roll_with_seed(expression: &str, seed: [u8; 32]) -> Result<RollResult, DiceError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_with_positive_modifier() {
        let result = RollResult {
            total: 9,
            dice_rolls: vec![4, 2],
            modifier: 5,
        };

        assert_eq!(format!("{}", result), "[4, 2] + 5 = 9");
    }

    #[test]
    fn test_display_with_negative_modifier() {
        let result = RollResult {
            total: 9,
            dice_rolls: vec![1, 7, 3],
            modifier: -2,
        };
        assert_eq!(result.to_string(), "[1, 7, 3] - 2 = 9");
    }

    #[test]
    fn test_display_with_zero_modifier() {
        let result = RollResult {
            total: 18,
            dice_rolls: vec![18],
            modifier: 0,
        };
        assert_eq!(result.to_string(), "[18] = 18");
    }

    #[test]
    fn test_parse_dx() {
        let (remaining, request) = dice_result("d6").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.quantity, 1);
        assert_eq!(request.sides, 6);
        assert_eq!(request.modifier, 0);
    }

    #[test]
    fn test_parse_simple_adx() {
        let (remaining, request) = dice_result("2d6").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.quantity, 2);
        assert_eq!(request.sides, 6);
        assert_eq!(request.modifier, 0);
    }
    #[test]
    fn test_parse_with_positive_modifier() {
        let (remaining, request) = dice_result("2d6+5").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.quantity, 2);
        assert_eq!(request.sides, 6);
        assert_eq!(request.modifier, 5);
    }

    #[test]
    fn test_parse_with_negative_modifier() {
        let (remaining, request) = dice_result("2d6-5").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.quantity, 2);
        assert_eq!(request.sides, 6);
        assert_eq!(request.modifier, -5);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let (remaining, request) = dice_result(" 2d6 +5").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.quantity, 2);
        assert_eq!(request.sides, 6);
        assert_eq!(request.modifier, 5);
    }
}
