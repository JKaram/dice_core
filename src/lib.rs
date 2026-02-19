//! A library for parsing and rolling dice notation.
//!
//! Supports standard dice notation like `2d6`, `1d20+5`, or `1d20+2d6-3`.
//!
//! # Examples
//!
//! ```
//! use dice_core::roll;
//!
//! let result = roll("2d6+3").unwrap();
//! println!("{}", result); // e.g., "[4, 2] + 3 = 9"
//! ```

mod error;
mod model;
mod parser;

pub use error::{DiceComponent, DiceError};
pub use model::RollResult;
pub use parser::{DiceRequest, DiceTerm, DropKeep, dice_result};

use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

/// Rolls dice based on a dice notation expression.
///
/// # Examples
///
/// ```
/// use dice_core::roll;
///
/// let result = roll("2d6+3").unwrap();
/// assert_eq!(result.dice_rolls.len(), 2);
/// assert_eq!(result.modifier, 3);
/// ```
///
/// Multiple dice terms are supported:
///
/// ```
/// use dice_core::roll;
///
/// let result = roll("1d20+1d4").unwrap();
/// assert_eq!(result.dice_rolls.len(), 2);
/// ```
///
/// # Errors
///
/// Returns `DiceError` if:
/// - The expression syntax is invalid
/// - Dice quantity or sides exceed limits (1000 dice max, 100 sides max)
/// - Float values are used where integers are required
/// - Quantity or sides are below 1
pub fn roll(expression: &str) -> Result<RollResult, DiceError> {
    let request = parse_and_validate(expression)?;
    let mut rng = rand::rng();

    roll_dice_with_rng(&request, &mut rng)
}

/// Rolls dice with a deterministic seed for reproducible results.
///
/// Useful for testing or replay systems.
///
/// # Examples
///
/// ```
/// use dice_core::roll_with_seed;
///
/// let seed = [42u8; 32];
/// let result1 = roll_with_seed("2d6+5", seed).unwrap();
/// let result2 = roll_with_seed("2d6+5", seed).unwrap();
///
/// assert_eq!(result1.total, result2.total);
/// assert_eq!(result1.dice_rolls, result2.dice_rolls);
/// ```
///
/// # Errors
///
/// Returns `DiceError` under the same conditions as `roll`.
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
    let mut dropped_rolls: Option<Vec<i32>> = None;
    let mut total = request.modifier as i32;

    for term in &request.terms {
        let quantity = term.quantity as i32;
        let sides = term.sides as i32;

        let mut term_rolls: Vec<i32> = (0..quantity)
            .map(|_| rng.random_range(1..=sides))
            .collect();

        // Apply drop/keep logic
        let kept_rolls = if let Some(ref dk) = term.drop_keep {
            let (kept, dropped) = apply_drop_keep(&term_rolls, dk, term.is_subtracted);
            if !dropped.is_empty() {
                dropped_rolls.get_or_insert_with(Vec::new).extend(dropped);
            }
            kept
        } else {
            term_rolls
        };

        for roll in kept_rolls {
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
        dropped_rolls,
        modifier: request.modifier as i32,
    })
}

fn apply_drop_keep(rolls: &[i32], dk: &DropKeep, is_subtracted: bool) -> (Vec<i32>, Vec<i32>) {
    let mut sorted = rolls.to_vec();
    sorted.sort_unstable();

    let (kept, dropped) = match dk {
        DropKeep::KeepHighest(n) => {
            let n = *n as usize;
            let split_at = sorted.len().saturating_sub(n);
            let dropped = sorted[..split_at].to_vec();
            let kept = sorted[split_at..].to_vec();
            (kept, dropped)
        }
        DropKeep::KeepLowest(n) => {
            let n = *n as usize;
            let kept = sorted[..n.min(sorted.len())].to_vec();
            let dropped = sorted[n.min(sorted.len())..].to_vec();
            (kept, dropped)
        }
        DropKeep::DropHighest(n) => {
            let n = *n as usize;
            let keep_count = sorted.len().saturating_sub(n);
            let kept = sorted[..keep_count].to_vec();
            let dropped = sorted[keep_count..].to_vec();
            (kept, dropped)
        }
        DropKeep::DropLowest(n) => {
            let n = *n as usize;
            let dropped = sorted[..n.min(sorted.len())].to_vec();
            let kept = sorted[n.min(sorted.len())..].to_vec();
            (kept, dropped)
        }
    };

    // Apply subtraction sign to dropped rolls too
    let dropped = if is_subtracted {
        dropped.into_iter().map(|r| -r).collect()
    } else {
        dropped
    };

    (kept, dropped)
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
            dropped_rolls: None,
            modifier: 5,
        };

        assert_eq!(format!("{}", result), "[4, 2] + 5 = 9");
    }

    #[test]
    fn test_display_with_negative_modifier() {
        let result = RollResult {
            total: 9,
            dice_rolls: vec![1, 7, 3],
            dropped_rolls: None,
            modifier: -2,
        };
        assert_eq!(result.to_string(), "[1, 7, 3] - 2 = 9");
    }

    #[test]
    fn test_display_with_zero_modifier() {
        let result = RollResult {
            total: 18,
            dice_rolls: vec![18],
            dropped_rolls: None,
            modifier: 0,
        };
        assert_eq!(result.to_string(), "[18] = 18");
    }

    #[test]
    fn test_display_with_negative_rolls() {
        let result = RollResult {
            total: 7,
            dice_rolls: vec![5, 3, -1],
            dropped_rolls: None,
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

    #[test]
    fn test_keep_highest_rolling() {
        // 4d6kh3 with a seed - should roll 4 dice and keep highest 3
        let result = roll_with_seed("4d6kh3", [42; 32]).unwrap();
        
        // Should have exactly 3 rolls kept
        assert_eq!(result.dice_rolls.len(), 3);
        // Should have 1 dropped roll
        assert_eq!(result.dropped_rolls.as_ref().map(|d| d.len()), Some(1));
        // Total should be sum of kept rolls only
        let sum: i32 = result.dice_rolls.iter().sum();
        assert_eq!(result.total, sum);
    }

    #[test]
    fn test_keep_lowest_rolling() {
        let result = roll_with_seed("4d6kl2", [42; 32]).unwrap();
        
        assert_eq!(result.dice_rolls.len(), 2);
        assert_eq!(result.dropped_rolls.as_ref().map(|d| d.len()), Some(2));
    }

    #[test]
    fn test_drop_highest_rolling() {
        // 5d6dh2 = roll 5, drop highest 2, keep 3 lowest
        let result = roll_with_seed("5d6dh2", [42; 32]).unwrap();
        
        assert_eq!(result.dice_rolls.len(), 3);
        assert_eq!(result.dropped_rolls.as_ref().map(|d| d.len()), Some(2));
    }

    #[test]
    fn test_drop_lowest_rolling() {
        // 5d6dl2 = roll 5, drop lowest 2, keep 3 highest
        let result = roll_with_seed("5d6dl2", [42; 32]).unwrap();
        
        assert_eq!(result.dice_rolls.len(), 3);
        assert_eq!(result.dropped_rolls.as_ref().map(|d| d.len()), Some(2));
    }

    #[test]
    fn test_keep_higher_than_roll_count() {
        // Asking to keep 10 when only rolling 3 - should keep all 3
        let result = roll_with_seed("3d6kh10", [42; 32]).unwrap();
        
        assert_eq!(result.dice_rolls.len(), 3);
        // No dice were actually dropped, so dropped_rolls is None
        assert!(result.dropped_rolls.is_none() || result.dropped_rolls.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_display_with_dropped_rolls() {
        let result = RollResult {
            total: 12,
            dice_rolls: vec![5, 4, 3],
            dropped_rolls: Some(vec![1]),
            modifier: 0,
        };
        assert_eq!(result.to_string(), "[5, 4, 3] ~[1] = 12");
    }
}
