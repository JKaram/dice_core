# **dice\_core**

dice\_core is a lightweight, robust dice rolling library for Rust. It parses standard dice notation (e.g., 2d6+5) and generates random results, providing detailed output including individual die rolls and totals.

It uses nom for efficient parsing and rand for random number generation, with support for seeded rolls via rand\_chacha.

## **Features**

* **Standard Notation:** Supports AdX (quantity d sides) syntax.  
* **Modifiers:** Supports positive and negative modifiers (e.g., \+5, \-2).  
* **Detailed Results:** Returns the total sum, individual die faces, and the applied modifier.  
* **Seeded RNG:** Supports deterministic rolling using a 32-byte seed.  
* **Safe Parsing:** Handles whitespace, case-insensitivity (d or D), and prevents excessive values (max 1000 dice).

## **Usage**

### **1\. Basic Rolling**

Use the roll function to parse a string and get a random result.

use dice\_core::roll;

fn main() {  
    match roll("2d6+3") {  
        Ok(result) \=\> {  
            // The Display impl formats it nicely: "\[3, 5\] \+ 3 \= 11"  
            println\!("Result: {}", result);   
              
            // You can also access fields directly  
            println\!("Total: {}", result.total);  
            println\!("Rolls: {:?}", result.dice\_rolls);  
        }  
        Err(e) \=\> eprintln\!("Error: {}", e),  
    }  
}

### **2\. Deterministic (Seeded) Rolling**

Use roll\_with\_seed if you need reproducible results (e.g., for testing or replay systems).

use dice\_core::roll\_with\_seed;

fn main() {  
    let seed \= \[42; 32\]; // 32-byte array  
    let result \= roll\_with\_seed("1d20", seed).unwrap();  
      
    println\!("{}", result);  
}

## **Dice Notation**

The library accepts strings in the format: \[quantity\]d\[sides\]\[modifier\]

### **Valid Examples**

| String | Description |
| :---- | :---- |
| 2d6 | Roll two 6-sided dice. |
| d20 | Roll one 20-sided die (quantity defaults to 1). |
| 2d6+5 | Roll two 6-sided dice and add 5 to the total. |
| 3d8-2 | Roll three 8-sided dice and subtract 2 from the total. |
| 2D6 | Case insensitive ('d' or 'D'). |
| 2d6 \+ 5 | Whitespace is ignored. |

### **Invalid Examples**

The parser is strict about integer formats and trailing garbage to ensure accuracy.

| String | Reason |
| :---- | :---- |
| 1.5d6 | Decimals are not allowed in quantity. |
| 1d2.5 | Decimals are not allowed in die sides. |
| 0d6 | Quantity must be positive (1-1000). |
| 1d0 | Die sides must be positive. |
| 1001d6 | Quantity limit exceeded (max 1000). |
| 2d6 hello | Trailing text/garbage is not allowed. |
| 2d | Missing die sides. |

## **Error Handling**

The library returns a DiceError enum on failure, covering:

* InvalidFormat: Malformed strings or trailing garbage.  
* InvalidQuantity: Zero or negative dice count.  
* InvalidDieSize: Zero or negative die sides.  
* QuantityLimitExceeded: Requesting more than 1000 dice.