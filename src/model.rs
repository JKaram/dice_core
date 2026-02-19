use std::fmt;

/// The result of rolling dice.
///
/// Contains the total sum, individual rolls, and any modifier applied.
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
    /// All individual dice rolls. Subtracted dice appear as negative values.
    pub dice_rolls: Vec<i32>,
    /// The numeric modifier applied to the total (can be negative).
    pub modifier: i32,
}

impl fmt::Display for RollResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifier == 0 {
            write!(f, "{:?} = {}", self.dice_rolls, self.total)
        } else if self.modifier > 0 {
            write!(
                f,
                "{:?} + {} = {}",
                self.dice_rolls, self.modifier, self.total
            )
        } else {
            write!(
                f,
                "{:?} - {} = {}",
                self.dice_rolls, -self.modifier, self.total
            )
        }
    }
}
