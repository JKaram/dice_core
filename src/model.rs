use std::fmt;

/// The result of rolling dice.
///
/// Contains the total sum, individual rolls, dropped rolls, and any modifier applied.
///
/// # Examples
///
/// ```
/// use dice_core::roll;
///
/// let result = roll("2d6+3").unwrap();
/// println!("{}", result); // e.g., "[4, 2] + 3 = 9"
/// println!("Total: {}", result.total);
/// println!("Rolls: {:?}", result.dice_rolls);
/// ```
pub struct RollResult {
    /// The final total after summing all dice and applying the modifier.
    pub total: i32,
    /// All individual dice rolls that were kept. Subtracted dice appear as negative values.
    pub dice_rolls: Vec<i32>,
    /// Rolls that were dropped (e.g., from 4d6kh3). None if no dice were dropped.
    pub dropped_rolls: Option<Vec<i32>>,
    /// The numeric modifier applied to the total (can be negative).
    pub modifier: i32,
}

impl fmt::Display for RollResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rolls_str = if let Some(ref dropped) = self.dropped_rolls {
            if dropped.is_empty() {
                format!("{:?}", self.dice_rolls)
            } else {
                format!("{:?} ~{:?}", self.dice_rolls, dropped)
            }
        } else {
            format!("{:?}", self.dice_rolls)
        };

        if self.modifier == 0 {
            write!(f, "{} = {}", rolls_str, self.total)
        } else if self.modifier > 0 {
            write!(f, "{} + {} = {}", rolls_str, self.modifier, self.total)
        } else {
            write!(f, "{} - {} = {}", rolls_str, -self.modifier, self.total)
        }
    }
}
