# **dice\_core**

A robust, safe, and flexible dice rolling library for Rust. dice\_core parses standard dice notation (e.g., 2d6+5), validates input limits, and provides both standard random rolling and deterministic seeded rolling.

Designed with strict parsing rules to prevent ambiguity (e.g., rejecting floating point quantities) and detailed error reporting.

## **Features**

* **Standard Notation**: Supports NdS, NdS+M, NdS-M (e.g., 1d20, 2d6+4, 3d8-2).  
* **Strict Parsing**: Uses nom for parser combinators to ensure input is well-formed. Rejects invalid inputs like 1.5d6 or 2d0.  
* **Safe Validation**: Enforces reasonable limits (max 1000 dice, positive die sizes) to prevent resource exhaustion or panic.  
* **Seeded RNG**: Supports roll\_with\_seed using ChaCha20Rng for deterministic results (useful for replay systems, tests, or games).  
* **Detailed Errors**: Custom DiceError enum provides clear, context-aware feedback for parsing and validation failures.  
* **Rich Results**: Returns a RollResult struct containing the total, individual die rolls, and the applied modifier.

## **Installation**

Add this to your Cargo.toml:

\[dependencies\]  
dice\_core \= { path \= "." } \# Or git repo if published  
rand \= "0.8"

## **Usage**

### **Basic Rolling**

The roll function uses the thread-local random number generator.

use dice\_core::roll;

fn main() {  
    match roll("2d6+5") {  
        Ok(result) \=\> {  
            println\!("Total: {}", result.total);  
            println\!("Rolls: {:?}", result.dice\_rolls);  
            println\!("Formatted: {}", result); // e.g., "\[3, 5\] \+ 5 \= 13"  
        }  
        Err(e) \=\> eprintln\!("Error: {}", e),  
    }  
}

### **Deterministic (Seeded) Rolling**

Use roll\_with\_seed when you need reproducible results. This uses the ChaCha20 algorithm.

use dice\_core::roll\_with\_seed;

fn main() {  
    // 32-byte array for the seed  
    let seed \= \[42; 32\];   
      
    let result \= roll\_with\_seed("1d20", seed).unwrap();  
      
    // This will output the same result every time for the same seed  
    println\!("Seeded Roll: {}", result.total);  
}

## **Supported Notation**

| Expression | Description |
| :---- | :---- |
| d20 | Roll one 20-sided die. |
| 1d20 | Explicitly roll one 20-sided die. |
| 2d6 | Roll two 6-sided dice and sum them. |
| 2d6+5 | Roll two 6-sided dice, sum them, and add 5\. |
| 3d8-2 | Roll three 8-sided dice, sum them, and subtract 2\. |
| 2 d 6 | Whitespace is forgiving between components. |

**Note:** Floating point numbers (e.g., 1.5d6) are explicitly **rejected** to ensure rules strictness.

## **Error Handling**

The library exposes a DiceError enum to handle specific failure cases programmatically:

pub enum DiceError {  
    InvalidFormat(String),       // Syntax errors, trailing garbage, floats  
    InvalidQuantity(i32),        // Quantity \<= 0  
    InvalidDieSize(i32),         // Sides \<= 0  
    QuantityLimitExceeded(i32),  // Quantity \> 1000  
    ParseError(ParseIntError),   // Integer parsing failures  
}

## **Development**

Run tests to verify logic, strict parsing, and error messages:

cargo test

## **License**

MIT