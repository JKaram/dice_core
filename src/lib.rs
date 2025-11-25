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
    let request = parse_and_validate(expression)?;
    let mut rng = rand::rng();

    roll_dice_with_rng(&request, &mut rng)
}

pub fn roll_with_seed(expression: &str, seed: [u8; 32]) -> Result<RollResult, DiceError> {
    let request = parse_and_validate(expression)?;
    let mut rng = ChaCha20Rng::from_seed(seed);

    roll_dice_with_rng(&request, &mut rng)
}

fn parse_and_validate(expression: &str) -> Result<DiceRequest, DiceError> {
    let (remaining, request) = dice_result(expression)
        .map_err(|e| DiceError::InvalidFormat(format!("Syntax error: {}", e)))?;

    validate_remaining(&remaining)?;
    validate_request(&request)?;

    Ok(request)
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

fn validate_remaining(remaining: &str) -> Result<(), DiceError> {
    if !remaining.is_empty() {
        if remaining.starts_with('.') {
            return Err(DiceError::InvalidFormat(format!(
                "Decimals are not allowed. Found: '{}'",
                remaining
            )));
        }
        return Err(DiceError::InvalidFormat(format!(
            "Could not parse the end of the expression: '{}'",
            remaining
        )));
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
        let err = roll(expression).unwrap_err();

        assert!(matches!(err, DiceError::InvalidFormat(ref msg) if msg.contains("Syntax error")));
    }

    #[test]
    fn test_parse_with_sides_float() {
        let expression = "1d2.6";
        let err = roll(expression).unwrap_err();

        assert!(matches!(err, DiceError::InvalidFormat(ref msg) if msg.contains("Syntax error")));
    }

    #[test]
    fn test_error_message_trailing_garbage() {
        let err = roll("2d6 hello").unwrap_err();
        assert!(
            matches!(err, DiceError::InvalidFormat(ref msg) if msg.contains("Could not parse the end of the expression"))
        );
    }

    #[test]
    fn test_quantity_limit_exceeded() {
        let result = roll("1001d6");

        match result {
            Err(DiceError::QuantityLimitExceeded(q)) => assert_eq!(q, 1001),
            _ => panic!("Expected QuantityLimitExceeded(1001), got {:?}", result),
        }
    }

    #[test]
    fn test_quantity_limit_boundary_ok() {
        let result = roll("1000d6");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_quantity_zero() {
        let result = roll("0d6");

        match result {
            Err(DiceError::InvalidQuantity(q)) => assert_eq!(q, 0),
            _ => panic!("Expected InvalidQuantity(0), got {:?}", result),
        }
    }

    #[test]
    fn test_invalid_die_size_zero() {
        let result = roll("1d0");

        match result {
            Err(DiceError::InvalidDieSize(s)) => assert_eq!(s, 0),
            _ => panic!("Expected InvalidDieSize(0), got {:?}", result),
        }
    }
}
