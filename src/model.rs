use std::fmt;

pub struct RollResult {
    pub total: i32,
    pub dice_rolls: Vec<i32>,
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
