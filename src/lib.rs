mod error;
mod model;
mod parser;

pub use error::DiceError;
pub use model::RollResult;
pub use parser::{DiceRequest, dice_result};

use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

pub fn roll(expression: &str) -> Result<RollResult, DiceError> {
    let (remaining, request) = dice_result(expression)
        .map_err(|e| DiceError::InvalidFormat(format!("Parse failed: {}", e)))?;

    if !remaining.is_empty() {
        return Err(DiceError::InvalidFormat("Unexpected input".to_string()));
    }

    validate_request(&request)?;

    let mut rng = rand::rng();

    roll_dice_with_rng(&request, &mut rng)
}

pub fn roll_with_seed(expression: &str, seed: [u8; 32]) -> Result<RollResult, DiceError> {
    let (remaining, request) = dice_result(expression)
        .map_err(|_| DiceError::InvalidFormat("Parse failed".to_string()))?;

    if !remaining.is_empty() {
        return Err(DiceError::InvalidFormat("Unexpected input".to_string()));
    }

    validate_request(&request)?;

    let mut rng = ChaCha20Rng::from_seed(seed);

    roll_dice_with_rng(&request, &mut rng)
}

fn validate_request(request: &DiceRequest) -> Result<(), DiceError> {
    if request.quantity > 1000 {
        return Err(DiceError::QuantityLimitExceeded(request.quantity));
    }
    if request.quantity <= 0 {
        return Err(DiceError::InvalidQuantity(request.quantity));
    }
    if request.sides <= 0 {
        return Err(DiceError::InvalidDieSize(request.sides));
    }
    Ok(())
}

fn roll_dice_with_rng<R: Rng>(request: &DiceRequest, rng: &mut R) -> Result<RollResult, DiceError> {
    let mut dice_rolls = Vec::new();

    for _ in 0..request.quantity {
        let roll = rng.random_range(1..=request.sides);
        dice_rolls.push(roll);
    }

    let dice_sum: i32 = dice_rolls.iter().sum();
    let total = dice_sum + request.modifier;

    Ok(RollResult {
        total,
        dice_rolls,
        modifier: request.modifier,
    })
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

    #[test]
    fn test_parse_with_bad_operator() {
        let (remaining, request) = dice_result("2d1*5").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.quantity, 2);
        assert_eq!(request.sides, 0);
        assert_eq!(request.modifier, 5);
    }

    #[test]
    fn test_roll_with_seed() {
        let result = roll_with_seed("2d6+5", [42; 32]).unwrap();
        assert_eq!(result.total, 8);
        assert_eq!(result.dice_rolls.len(), 2);
        assert_eq!(result.modifier, 5);
    }

    #[test]
    fn test_parse_with_quantity_float() {
        let expression = "1.5d6";
        let result = roll(expression);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_sides_float() {
        let expression = "1d2.6";
        let result = roll(expression);

        assert!(result.is_err())
    }
}
