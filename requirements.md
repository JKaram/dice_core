# `dice_core` Crate Requirements

This document outlines the technical requirements, public API, and usage examples for the `dice_core` Rust crate.

## 🎯 Project Goal

To create a high-performance, easy-to-use, and extensible Rust library for parsing and rolling dice notation.

## ⚙️ Public API (Exports)

The crate will export the following four items from its library root:

1.  **`roll(expression: &str) -> Result<RollResult, DiceError>`**
    * The primary function for non-deterministic (random) dice rolls.

2.  **`roll_with_seed(expression: &str, seed: [u8; 32]) -> Result<RollResult, DiceError>`**
    * A deterministic function for testing and verifiable rolls. It will always produce the same result given the same seed.

3.  **`struct RollResult`**
    * The data structure returned on a successful roll.
    * **Fields:**
        * `total: i32`: The final, calculated total (dice rolls + modifier).
        * `dice_rolls: Vec<i32>`: A vector containing the individual outcome of **each die** rolled. For example, a `3d6` roll might produce `[2, 5, 1]`.
        * `modifier: i32`: The positive or negative number added to the sum of the dice.
            * For `1d20+5`, this would be `5`.
            * For `3d8-2`, this would be `-2`.
            * For `1d6`, this would be `0`.

4.  **`enum DiceError`**
    * A custom error enum (using `thiserror`) that describes all possible failures.
    * **Variants:**
        * `InvalidFormat(String)`: For general parsing failures on invalid notation.
        * `InvalidQuantity(i32)`: For `0` dice.
        * `InvalidDieSize(i32)`: For `d0` or `d-1`.
        * `QuantityLimitExceeded(i32)`: For rolls over the 1,000 die limit.
        * `ParseError(std::num::ParseIntError)`: For numbers that are too large.

### `RollResult` Behavior: Display Trait

The `RollResult` struct must implement the `std::fmt::Display` trait to provide a "pretty-print" format. This allows users of the crate to print the result directly.

**Desired Output:**

* **For `2d6+5` (Result: `[4, 2]`, Mod: `5`, Total: `11`):**
    `[4, 2] + 5 = 11`
* **For `3d8-2` (Result: `[1, 7, 3]`, Mod: `-2`, Total: `9`):**
    `[1, 7, 3] - 2 = 9`
* **For `1d20` (Result: `[18]`, Mod: `0`, Total: `18`):**
    `[18] = 18`

---

## 🎲 Expression Parsing

### Parsing Strategy: Parser Combinator

This project will use a formal parser library (such as **`nom`** or **`pest`**) instead of regular expressions.

**Rationale:**
* **Extensibility:** Even though the MVP scope is simple, using a real parser provides a robust foundation. This architecture will easily scale to support complex math, operator precedence, and nested expressions (e.g., `(1d20+5)/2`) in the future without a complete rewrite.
* **Maintainability:** Parser combinators are more readable and maintainable than complex regex.
* **Error Reporting:** A parser library provides a better framework for generating meaningful, human-readable errors.

**MVP Scope:**
For this MVP, the parser will *only* be implemented to accept the simple `AdX[+/-C]` format. All other complex notation (like `1d20+1d4` or `(1d6)*2`) will be rejected as an `InvalidFormat` error, as defined in the "Out of Scope" section.

### ✅ Valid Expressions (Should Succeed)

The parser must correctly handle these formats. Whitespace should be handled gracefully.

| Expression | Notation Type | Notes |
| :--- | :--- | :--- |
| `1d20` | `AdX` | A single 20-sided die. |
| `2d6` | `AdX` | Two 6-sided dice. |
| `d6` | `dX` (Implied 'A') | An implied "1" for the quantity. Parses as `1d6`. |
| `d20` | `dX` (Implied 'A') | Parses as `1d20`. |
| `1d20+5` | `AdX+C` | Standard roll with a positive modifier. |
| `3d8-2` | `AdX-C` | Standard roll with a negative modifier. |
| `d4+1` | `dX+C` | Implied "1" die with a positive modifier. |
| `d12-3` | `dX-C` | Implied "1" die with a negative modifier. |
| `1000d6`| `AdX` | The maximum allowed quantity. |
| `1d20 + 5` | Whitespace | Parser must ignore surrounding/internal whitespace. |
| ` 2d6 - 1 ` | Whitespace | Parser must ignore surrounding/internal whitespace. |

### ❌ Invalid Expressions (Should Return `DiceError`)

The parser must reject these formats and return a descriptive `DiceError`.

| Expression | Reason for Failure (Expected Error) |
| :--- | :--- | :--- |
| `1001d6` | **Invalid Quantity:** Exceeds the 1,000 die limit. (`QuantityLimitExceeded`) |
| `1d20*5` | **Invalid Operator:** Only `+` and `-` supported. (`InvalidFormat`) |
| `2d6+1d4` | **Invalid Notation:** Only one dice term allowed. (`InvalidFormat`) |
| `1d` | **Invalid Format:** Missing the die type. (`InvalidFormat`) |
| `1d0` | **Invalid Die:** A "0-sided" die is impossible. (`InvalidDieSize`) |
| `0d20` | **Invalid Quantity:** Must roll at least one die. (`InvalidQuantity`) |
| `1.5d6` | **Invalid Quantity:** Floats not allowed. (`InvalidFormat`) |

---

## ⛔ Out of Scope (Future Features)

The following features are explicitly **not** part of this MVP and should be rejected by the parser (likely with `DiceError::InvalidFormat`):

* **High-Volume Rolls:** The crate will enforce a hard limit of **1,000** dice per roll (e.g., `1000d6`). Any quantity above this must return a `DiceError::QuantityLimitExceeded`.
* **Multiple Dice Terms:** (e.g., `1d20+1d4`)
* **Complex Math:** (e.g., `(1d20+5)/2`, `1d20*10`)
* **"Drop/Keep" Notation:** (e.g., `4d6kh3` - "roll 4 d6, keep highest 3")
* **Specialized Dice:** (e.g., `dF` - Fate/Fudge dice)
* **Exploding Dice:** (e.g., `1d6!`)

---

## 🚀 Example Usage

This is how a developer would use the `dice_core` crate in their own Rust application.

```rust
// main.rs
use dice_core::{roll, RollResult, DiceError};

fn main() {
    println!("--- Crate Usage Example ---");

    // --- 1. A simple, successful roll ---
    let expression1 = "2d6+5";
    println!("\nAttempting to roll: {}", expression1);

    match roll(expression1) {
        Ok(result) => {
            println!("✅ Success!");
            println!("   - Total: {}", result.total);
            println!("   - Individual Dice: {:?}", result.dice_rolls); // e.g., [3, 4]
            println!("   - Modifier: {}", result.modifier); // e.g., 5
            
            // Example of using the Display trait
            println!("   - Pretty Output: '{}'", result); // e.g., '[3, 4] + 5 = 12'
        }
        Err(e) => {
            println!("❌ Error rolling dice: {}", e);
        }
    }

    // --- 2. A roll that will fail (Invalid Operator) ---
    let expression2 = "1d20*2"; // Not in MVP scope
    println!("\nAttempting to roll: {}", expression2);

    if let Err(e) = roll(expression2) {
        println!("❌ Error as expected: {}", e);
    }
    
    // --- 3. A roll that will fail (Quantity Limit) ---
    let expression3 = "1001d20";
    println!("\nAttempting to roll: {}", expression3);

    if let Err(e) = roll(expression3) {
        println!("❌ Error as expected: {}", e);
    }

    // --- 4. A deterministic (seeded) roll for testing ---
    let expression4 = "1d20+7";
    let seed = [42; 32]; // A 32-byte array
    println!("\nAttempting a deterministic roll: {}", expression4);

    if let Ok(result) = dice_core::roll_with_seed(expression4, seed) {
        println!("✅ Success! This total will always be the same.");
        println!("   - Total: {}", result.total);
    }
}
```