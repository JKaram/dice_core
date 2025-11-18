# Project Plan: Incremental PRs

This document outlines the 10-step plan for building the `dice_core` library incrementally. Each step represents a single, focused Pull Request.

---

### 🚀 PR 1: Project Scaffolding & API Definition

**Goal:** Set up the empty library, define the public API (the "shape" of your functions and types), and add dependencies. Nothing will *work*, but the crate will compile.

**What to do:**
* Run `cargo new dice_core --lib`.
* Create your file structure: `src/lib.rs`, `src/error.rs`, `src/model.rs`, `src/parser.rs`.
* Add all dependencies to `Cargo.toml` (`thiserror`, `rand`, `rand_chacha`, `rand_core`, `nom`).
* In `src/model.rs`, define the `pub struct RollResult` with its fields.
* In `src/error.rs`, define the `pub enum DiceError` with all its variants using `#[derive(Error)]`.
* In `src/lib.rs`, define the *empty* functions. Make them return a hard-coded error or use the `todo!()` macro.
    ```rust
    // In src/lib.rs
    pub use error::DiceError;
    pub use model::RollResult;
    
    pub fn roll(expression: &str) -> Result<RollResult, DiceError> {
        // We'll implement this later
        Err(DiceError::InvalidFormat("Not implemented".to_string()))
    }
    
    pub fn roll_with_seed(expression: &str, seed: [u8; 32]) -> Result<RollResult, DiceError> {
        // We'll implement this later
        todo!()
    }
    ```
**What to test:** Run `cargo build`. If it compiles without errors, this PR is done.

---

### 🎨 PR 2: Implement `fmt::Display` for `RollResult`

**Goal:** Make your `RollResult` struct printable, exactly as your requirements specify.

**What to do:**
* In `src/model.rs`, add `use std::fmt;`.
* Implement `impl fmt::Display for RollResult { ... }`.
* Write the logic to handle the three cases: positive modifier, negative modifier, and zero modifier.

**What to test:** Add a unit test module at the bottom of `src/model.rs`.
```rust
// In src/model.rs
#[cfg(test)]
mod tests {
    use super::*; // Import RollResult

    #[test]
    fn test_display_with_positive_modifier() {
        let result = RollResult { total: 11, dice_rolls: vec![4, 2], modifier: 5 };
        assert_eq!(result.to_string(), "[4, 2] + 5 = 11");
    }

    #[test]
    fn test_display_with_negative_modifier() {
        let result = RollResult { total: 9, dice_rolls: vec![1, 7, 3], modifier: -2 };
        assert_eq!(result.to_string(), "[1, 7, 3] - 2 = 9");
    }

    #[test]
    fn test_display_with_no_modifier() {
        let result = RollResult { total: 18, dice_rolls: vec![18], modifier: 0 };
        assert_eq!(result.to_string(), "[18] = 18");
    }
}