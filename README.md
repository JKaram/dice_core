# **dice_core**

dice_core parses standard dice notation (e.g., 2d6+5) and generates random results, providing detailed output including individual die rolls and totals.

## **Features**

* **Standard Notation:** Supports AdX (quantity d sides) syntax.  
* **Modifiers:** Supports positive and negative modifiers (e.g., +5, -2).  
* **Detailed Results:** Returns the total sum, individual die faces, and the applied modifier.  
* **Seeded RNG:** Supports deterministic rolling using a 32-byte seed.  
* **Safe Parsing:** Handles whitespace, case-insensitivity (d or D), and prevents excessive values (max 1000 dice).

## **Usage**

### **1. Basic Rolling**

Use the roll function to parse a string and get a random result.

```rust
use dice_core::roll;

fn main() {  
    match roll("2d6+3") {  
        Ok(result) => {  
            // The Display impl formats it nicely: "[3, 5] + 3 = 11"  
            println!("Result: {}", result);   
              
            // You can also access fields directly  
            println!("Total: {}", result.total);  
            println!("Rolls: {:?}", result.dice_rolls);  
        }  
        Err(e) => eprintln!("Error: {}", e),  
    }  
}
```

### **2. Deterministic Rolling**

Use `roll_with_seed` if you need reproducible results (e.g. for testing or replay systems).

```rust
use dice_core::roll_with_seed;

fn main() {  
    let seed = [42; 32]; // 32-byte array  
    let result = roll_with_seed("1d20", seed).unwrap();  
      
    println!("{}", result);  
}
```

## **The RollResult Struct**

On a successful roll, the library returns a `RollResult` struct containing detailed information about the operation.

```rust
pub struct RollResult {  
    pub total: i32,  
    pub dice_rolls: Vec<i32>,  
    pub modifier: i32,  
}
```

The struct implements `std::fmt::Display`, allowing you to print it directly to get a readable equation string (e.g., `[4, 2] + 5 = 11)`).

## **Dice Notation**

The library accepts strings in the format: `[quantity]d[sides][modifier]`

### **Valid Examples**

| String      | Description                                      |
| :---------- | :----------------------------------------------- |
| `2d6`       | Roll two 6-sided dice.                           |
| `d20`       | Roll one 20-sided die (quantity defaults to 1).  |
| `2d6+5`     | Roll two 6-sided dice and add 5 to the total.    |
| `3d8-2`     | Roll three 8-sided dice and subtract 2 from the total. |
| `2D6`       | Case insensitive ('d' or 'D').                   |
| `2d6 + 5`   | Whitespace is ignored.                           |

### Error Handling

The library uses a custom `DiceError` enum to handle parsing and validation failures. Errors are categorized by the type of violation (e.g., floating point usage, limits exceeded).

#### Error Types

* **`DiceError::InvalidFormat(String)`**: Returned when the parser cannot understand the syntax, or if there is trailing garbage after a valid expression.
* **`DiceError::FloatParseError(DiceComponent, f64)`**: Returned when a decimal is used where an integer is required (quantity, sides, or modifiers).
* **`DiceError::LimitExceeded(DiceComponent, i32)`**: Returned when a value exceeds the maximum allowed limit (Max Quantity: 1000, Max Sides: 100).
* **`DiceError::BelowMinimum(DiceComponent, i32)`**: Returned when a value is too low (Quantity and Sides must be positive integers > 0).

#### Invalid Examples

| Expression | Error Variant | Component | Reason |
| :--- | :--- | :--- | :--- |
| `1.5d6` | `FloatParseError` | `Quantity` | Quantity cannot be a float. |
| `1d6.2` | `FloatParseError` | `Sides` | Die size cannot be a float. |
| `1d6+1.5` | `FloatParseError` | `Modifier` | Modifier cannot be a float. |
| `1001d6` | `LimitExceeded` | `Quantity` | Quantity cannot exceed 1000. |
| `1d101` | `LimitExceeded` | `Sides` | Die size cannot exceed 100. |
| `0d6` | `BelowMinimum` | `Quantity` | Quantity must be at least 1. |
| `1d0` | `BelowMinimum` | `Sides` | Die size must be at least 1. |
| `2d6 hello`| `InvalidFormat` | N/A | Expression contains trailing garbage. |

## **Future Features**

The following features are on the herizon:

* **High-Volume Rolls:** The crate will enforce a hard limit of **1,000** dice per roll (e.g., 1000d6).
* **Multiple Dice Terms:** (e.g., 1d20+1d4)  
* **Complex Math:** (e.g., (1d20+5)/2, 1d20*10)  
* **"Drop/Keep" Notation:** (e.g., 4d6kh3 - "roll 4 d6, keep highest 3")  
* **Specialized Dice:** (e.g., dF - Fate/Fudge dice)  
* **Exploding Dice:** (e.g., 1d6!)
