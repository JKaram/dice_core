mod error;
mod model;
mod parser;

pub use error::{DiceComponent, DiceError};
pub use model::RollResult;
pub use parser::{DiceRequest, DiceTerm, dice_result};

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
    let mut total_dice = 0;

    for term in &request.terms {
        if term.quantity.fract() != 0.0 {
            return Err(DiceError::FloatParseError(
                DiceComponent::Quantity,
                term.quantity,
            ));
        }
        if term.sides.fract() != 0.0 {
            return Err(DiceError::FloatParseError(
                DiceComponent::Sides,
                term.sides,
            ));
        }

        let quantity = term.quantity as i32;
        let sides = term.sides as i32;

        if quantity <= 0 {
            return Err(DiceError::BelowMinimum(DiceComponent::Quantity, quantity));
        }
        if sides <= 0 {
            return Err(DiceError::BelowMinimum(DiceComponent::Sides, sides));
        }
        if sides > 100 {
            return Err(DiceError::LimitExceeded(DiceComponent::Sides, sides));
        }

        total_dice += quantity;
    }

    if total_dice > 1000 {
        return Err(DiceError::LimitExceeded(DiceComponent::Quantity, total_dice));
    }

    if request.modifier.fract() != 0.0 {
        return Err(DiceError::FloatParseError(
            DiceComponent::Modifier,
            request.modifier,
        ));
    }

    Ok(())
}

fn validate_remaining(remaining: &str) -> Result<(), DiceError> {
    if !remaining.is_empty() {
        return Err(DiceError::InvalidFormat(format!(
            "Could not parse the end of the expression: '{}'",
            remaining
        )));
    }

    Ok(())
}

fn roll_dice_with_rng<R: Rng>(request: &DiceRequest, rng: &mut R) -> Result<RollResult, DiceError> {
    let mut dice_rolls = Vec::new();
    let mut total = request.modifier as i32;

    for term in &request.terms {
        let quantity = term.quantity as i32;
        let sides = term.sides as i32;

        for _ in 0..quantity {
            let roll = rng.random_range(1..=sides);
            if term.is_subtracted {
                dice_rolls.push(-roll);
                total -= roll;
            } else {
                dice_rolls.push(roll);
                total += roll;
            }
        }
    }

    Ok(RollResult {
        total,
        dice_rolls,
        modifier: request.modifier as i32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_term() {
        let result = roll_with_seed("2d6+3", [42; 32]).unwrap();
        assert_eq!(result.dice_rolls.len(), 2);
        assert_eq!(result.modifier, 3);
    }

    #[test]
    fn test_two_terms() {
        let result = roll_with_seed("1d4+1d6", [42; 32]).unwrap();
        assert_eq!(result.dice_rolls.len(), 2);
    }

    #[test]
    fn test_subtract_term() {
        let result = roll_with_seed("2d6-1d4", [42; 32]).unwrap();
        assert_eq!(result.dice_rolls.len(), 3);
        assert!(result.dice_rolls[2] < 0);
    }

    #[test]
    fn test_mixed_terms_and_modifier() {
        let result = roll_with_seed("1d20+2d6-5", [42; 32]).unwrap();
        assert_eq!(result.dice_rolls.len(), 3);
        assert_eq!(result.modifier, -5);
    }

    #[test]
    fn test_three_terms() {
        let result = roll_with_seed("1d4+1d6+1d8", [42; 32]).unwrap();
        assert_eq!(result.dice_rolls.len(), 3);
    }

    #[test]
    fn test_total_dice_limit() {
        let result = roll("1000d6");
        assert!(result.is_ok());

        let result = roll("1001d6");
        assert!(matches!(result, Err(DiceError::LimitExceeded(DiceComponent::Quantity, 1001))));

        let result = roll("500d6+501d4");
        assert!(matches!(result, Err(DiceError::LimitExceeded(DiceComponent::Quantity, 1001))));
    }

    #[test]
    fn test_display_with_positive_modifier() {
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
    fn test_display_with_negative_rolls() {
        let result = RollResult {
            total: 7,
            dice_rolls: vec![5, 3, -1],
            modifier: 0,
        };
        assert_eq!(result.to_string(), "[5, 3, -1] = 7");
    }

    #[test]
    fn test_parse_simple() {
        let (remaining, request) = dice_result("2d6").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.terms.len(), 1);
        assert_eq!(request.terms[0].quantity, 2.0);
        assert_eq!(request.terms[0].sides, 6.0);
        assert!(!request.terms[0].is_subtracted);
    }

    #[test]
    fn test_parse_dx() {
        let (remaining, request) = dice_result("d6").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.terms[0].quantity, 1.0);
        assert_eq!(request.terms[0].sides, 6.0);
    }

    #[test]
    fn test_parse_with_positive_modifier() {
        let (remaining, request) = dice_result("2d6+5").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.terms.len(), 1);
        assert_eq!(request.modifier, 5.0);
    }

    #[test]
    fn test_parse_with_negative_modifier() {
        let (remaining, request) = dice_result("2d6-5").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.modifier, -5.0);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let (remaining, request) = dice_result(" 2d6 +5").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.terms[0].quantity, 2.0);
        assert_eq!(request.modifier, 5.0);
    }

    #[test]
    fn test_parse_two_terms() {
        let (remaining, request) = dice_result("1d20+1d4").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(request.terms.len(), 2);
        assert_eq!(request.terms[0].sides, 20.0);
        assert_eq!(request.terms[1].sides, 4.0);
    }

    #[test]
    fn test_parse_with_bad_operator() {
        let (remaining, request) = dice_result("2d1*5").unwrap();
        assert_eq!(remaining, "*5");
        assert_eq!(request.terms[0].quantity, 2.0);
        assert_eq!(request.terms[0].sides, 1.0);
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
        let err = roll(expression);

        match err {
            Err(DiceError::FloatParseError(DiceComponent::Quantity, val)) => assert_eq!(val, 1.5),
            _ => panic!("Expected FloatParseError(Quantity, 1.5)"),
        }
    }

    #[test]
    fn test_parse_with_sides_float() {
        let expression = "1d2.6";
        let err = roll(expression);

        match err {
            Err(DiceError::FloatParseError(DiceComponent::Sides, val)) => assert_eq!(val, 2.6),
            _ => panic!("Expected FloatParseError(Sides, 2.6)"),
        }
    }

    #[test]
    fn test_parse_with_modifier_float() {
        let expression = "1d6+2.5";
        let err = roll(expression);

        match err {
            Err(DiceError::FloatParseError(DiceComponent::Modifier, val)) => assert_eq!(val, 2.5),
            _ => panic!("Expected FloatParseError(Modifier, 2.5)"),
        }
    }

    #[test]
    fn test_error_message_trailing_garbage() {
        let err = roll("2d6 hello");
        assert!(
            matches!(err, Err(DiceError::InvalidFormat(ref msg)) if msg.contains("Could not parse the end of the expression"))
        );
    }

    #[test]
    fn test_sides_limit_exceeded() {
        let result = roll("1d101");

        match result {
            Err(DiceError::LimitExceeded(DiceComponent::Sides, s)) => assert_eq!(s, 101),
            _ => panic!("Expected LimitExeeded(101)"),
        }
    }

    #[test]
    fn test_quantity_limit_exceeded_single_term() {
        let result = roll("1001d6");

        match result {
            Err(DiceError::LimitExceeded(DiceComponent::Quantity, q)) => assert_eq!(q, 1001),
            _ => panic!("Expected LimitExeeded(1001)"),
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
            Err(DiceError::BelowMinimum(DiceComponent::Quantity, q)) => assert_eq!(q, 0),
            _ => panic!("Expected InvalidQuantity(0)"),
        }
    }

    #[test]
    fn test_invalid_die_size_zero() {
        let result = roll("1d0");

        match result {
            Err(DiceError::BelowMinimum(DiceComponent::Sides, s)) => assert_eq!(s, 0),
            _ => panic!("Expected InvalidDieSize(0)"),
        }
    }
}
