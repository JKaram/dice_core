# **dice_core**

dice_core is a lightweight, robust dice rolling library for Rust. It parses standard dice notation (e.g., 2d6+5) and generates random results, providing detailed output including individual die rolls and totals.

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

### **Invalid Examples**

The parser enforces integer-only values and checks boundaries.

| String      | Reason                                 | Error Type              |
| :---------- | :------------------------------------- | :---------------------- |
| `1.5d6`     | Decimals are not allowed in quantity.  | `FloatQuantity`         |
| `1d2.5`     | Decimals are not allowed in die sides. | `FloatDieSize`          |
| `1d6+0.5`   | Decimals are not allowed in modifiers. | `FloatModifier`         |
| `0d6`       | Quantity must be positive (1-1000).    | `InvalidQuantity`       |
| `1d0`       | Die sides must be positive.            | `InvalidDieSize`        |
| `1001d6`    | Quantity limit exceeded (max 1000).    | `QuantityLimitExceeded` |
| `2d6 hello` | Trailing text/garbage is not allowed.  | `InvalidFormat`         |

## **Error Handling**

The library returns a `DiceError` enum on failure. This allows you to match on specific error cases if needed.

* **InvalidFormat**: Malformed strings or trailing garbage characters.  
* **InvalidQuantity**: Zero or negative dice count.  
* **InvalidDieSize**: Zero or negative die sides.  
* **QuantityLimitExceeded**: Requesting more than 1000 dice.  
* **FloatParseError**: Floats will be rejected.
* **ParseError**: Generic catch for all other errors.

## **Future Features**

The following features are on the herizon:

* **High-Volume Rolls:** The crate will enforce a hard limit of **1,000** dice per roll (e.g., 1000d6).
* **Multiple Dice Terms:** (e.g., 1d20+1d4)  
* **Complex Math:** (e.g., (1d20+5)/2, 1d20*10)  
* **"Drop/Keep" Notation:** (e.g., 4d6kh3 - "roll 4 d6, keep highest 3")  
* **Specialized Dice:** (e.g., dF - Fate/Fudge dice)  
* **Exploding Dice:** (e.g., 1d6!)
