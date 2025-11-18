# **Project Plan: Incremental PRs**

This document outlines the 10-step plan for building the dice\_core library incrementally. Each step represents a single, focused Pull Request.

### **🚀 PR 1: Project Scaffolding & API Definition**

**Goal:** Set up the empty library, define the public API (the "shape" of your functions and types), and add dependencies. Nothing will *work*, but the crate will compile.

**What to do:**

* Run cargo new dice\_core \--lib.  
* Create your file structure: src/lib.rs, src/error.rs, src/model.rs, src/parser.rs.  
* Add all dependencies to Cargo.toml (thiserror, rand, rand\_chacha, rand\_core, nom).  
* In src/model.rs, define the pub struct RollResult with its fields.  
* In src/error.rs, define the pub enum DiceError with all its variants using \#\[derive(Error)\].  
* In src/lib.rs, define the *empty* functions. Make them return a hard-coded error or use the todo\!() macro.  
  // In src/lib.rs  
  pub use error::DiceError;  
  pub use model::RollResult;

  pub fn roll(expression: \&str) \-\> Result\<RollResult, DiceError\> {  
      // We'll implement this later  
      Err(DiceError::InvalidFormat("Not implemented".to\_string()))  
  }

  pub fn roll\_with\_seed(expression: \&str, seed: \[u8; 32\]) \-\> Result\<RollResult, DiceError\> {  
      // We'll implement this later  
      todo\!()  
  }

**What to test:** Run cargo build. If it compiles without errors, this PR is done.

### **🎨 PR 2: Implement fmt::Display for RollResult**

**Goal:** Make your RollResult struct printable, exactly as your requirements specify.

**What to do:**

* In src/model.rs, add use std::fmt;.  
* Implement impl fmt::Display for RollResult { ... }.  
* Write the logic to handle the three cases: positive modifier, negative modifier, and zero modifier.

**What to test:** Add a unit test module at the bottom of src/model.rs.

// In src/model.rs  
\#\[cfg(test)\]  
mod tests {  
    use super::\*; // Import RollResult

    \#\[test\]  
    fn test\_display\_with\_positive\_modifier() {  
        let result \= RollResult { total: 11, dice\_rolls: vec\!\[4, 2\], modifier: 5 };  
        assert\_eq\!(result.to\_string(), "\[4, 2\] \+ 5 \= 11");  
    }

    \#\[test\]  
    fn test\_display\_with\_negative\_modifier() {  
        let result \= RollResult { total: 9, dice\_rolls: vec\!\[1, 7, 3\], modifier: \-2 };  
        assert\_eq\!(result.to\_string(), "\[1, 7, 3\] \- 2 \= 9");  
    }

    \#\[test\]  
    fn test\_display\_with\_no\_modifier() {  
        let result \= RollResult { total: 18, dice\_rolls: vec\!\[18\], modifier: 0 };  
        assert\_eq\!(result.to\_string(), "\[18\] \= 18");  
    }  
}

Run cargo test. If your new tests pass, merge it.

### **🧩 PR 3: Parser \- The Core AdX (e.g., 2d6)**

**Goal:** Start the parser. We will *only* parse the AdX format. No modifiers, no implied dice.

**What to do:**

* In src/parser.rs, create a *private* helper struct to hold the parsed data, e.g., struct DiceRequest { quantity: i32, sides: i32, modifier: i32 }.  
* Create a nom parser that *only* parses "2d6" or "1d20".  
* It should parse a number (A), then the literal d, then another number (X).  
* If it succeeds, return Ok(DiceRequest { quantity: A, sides: X, modifier: 0 }).

**What to test:** Add unit tests in src/parser.rs that *only* test the parser.

// In src/parser.rs (at the bottom)  
\#\[cfg(test)\]  
mod tests {  
    use super::\*; // Import your parser function

    \#\[test\]  
    fn test\_parse\_simple\_adx() {  
        let (remaining, request) \= your\_parser\_function("2d6").unwrap();  
        assert\!(remaining.is\_empty());  
        assert\_eq\!(request.quantity, 2);  
        assert\_eq\!(request.sides, 6);  
        assert\_eq\!(request.modifier, 0);  
    }  
      
    \#\[test\]  
    fn test\_parse\_fails\_on\_modifier() {  
        // We haven't taught it this yet\!  
        assert\!(your\_parser\_function("2d6+5").is\_err());  
    }  
}

### **➕ PR 4: Parser \- Add Modifiers (e.g., 2d6+5)**

**Goal:** Teach the parser to *optionally* look for a \+ or \- and another number.

**What to do:**

* Modify your nom parser from PR 3\.  
* After parsing AdX, use nom::combinator::opt to look for an optional part.  
* That optional part should parse a \+ or \- character, then a number.  
* Update the DiceRequest to include this modifier.

**What to test:** Add *new* tests to src/parser.rs.

// In src/parser.rs (in the test module)  
\#\[test\]  
fn test\_parse\_with\_modifier() {  
    let (rem, req) \= your\_parser\_function("2d6+5").unwrap();  
    assert\!(rem.is\_empty());  
    assert\_eq\!(req.modifier, 5);

    let (rem, req) \= your\_parser\_function("1d20-3").unwrap();  
    assert\!(rem.is\_empty());  
    assert\_eq\!(req.modifier, \-3);  
      
    // Make sure the old test still passes\!  
    let (rem, req) \= your\_parser\_function("3d8").unwrap();  
    assert\!(rem.is\_empty());  
    assert\_eq\!(req.modifier, 0);  
}

### **🎲 PR 5: Parser \- Add Implied 1 (e.g., d20)**

**Goal:** Teach the parser to handle dX (like d20 or d6+1) by making the "A" quantity optional.

**What to do:**

* Modify your parser again.  
* Use nom::combinator::opt on the *first* number (the quantity).  
* If it's missing (None), default it to 1\.

**What to test:** Add *new* tests.

// In src/parser.rs (in the test module)  
\#\[test\]  
fn test\_parse\_implied\_quantity() {  
    let (rem, req) \= your\_parser\_function("d20").unwrap();  
    assert\!(rem.is\_empty());  
    assert\_eq\!(req.quantity, 1);  
    assert\_eq\!(req.sides, 20);

    let (rem, req) \= your\_parser\_function("d6+2").unwrap();  
    assert\!(rem.is\_empty());  
    assert\_eq\!(req.quantity, 1);  
    assert\_eq\!(req.modifier, 2);  
}

### **🧹 PR 6: Parser \- Add Whitespace Handling**

**Goal:** Make your parser ignore whitespace (e.g., 2d6 \+ 5).

**What to do:**

* nom has special "combinators" for this. You'll want to wrap your individual parsers (for numbers, the d, the \+) in a whitespace-aware combinator like nom::character::complete::multispace0.

**What to test:** Add *new* tests.

// In src/parser.rs (in the test module)  
\#\[test\]  
fn test\_parse\_with\_whitespace() {  
    let (rem, req) \= your\_parser\_function(" 2d6 \+ 5 ").unwrap();  
    assert\!(rem.is\_empty());  
    assert\_eq\!(req.quantity, 2);  
    assert\_eq\!(req.sides, 6);  
    assert\_eq\!(req.modifier, 5);  
}

### **🎉 PR 7: Implement roll() Logic ("Happy Path")**

**Goal:** Connect everything\! Make the public roll() function *actually* parse and roll dice, but only for valid inputs.

**What to do:**

* In src/lib.rs, update roll().  
* Call your parser. If it fails, map the nom error to your DiceError::InvalidFormat and return it.  
* If it succeeds (Ok(DiceRequest)), create a rand::thread\_rng().  
* Loop request.quantity times, using rng.gen\_range(1..=request.sides) for each die.  
* Collect the results into a Vec\<i32\>.  
* Calculate the total.  
* Return Ok(RollResult { ... }).

**What to test:** This is hard to test perfectly because it's random. For this PR, you'll mainly test it "by eye" using the examples/ directory (which you'll make in the last PR). You *can* add a simple test that just checks if it returns Ok.

### **🔒 PR 8: Implement roll\_with\_seed()**

**Goal:** Implement the deterministic roll\_with\_seed function. This will also make your logic testable\!

**What to do:**

* In src/lib.rs, update roll\_with\_seed().  
* It will be 99% copy-paste of roll(). It should call the *same parser*.  
* The *only* difference is the RNG. Instead of rand::thread\_rng(), use:  
  use rand\_core::SeedableRng;  
  let mut rng \= rand\_chacha::ChaCha8Rng::from\_seed(seed);

**What to test:** Now you can write a perfect, deterministic test\!

// In src/lib.rs (in a test module)  
\#\[test\]  
fn test\_seeded\_roll() {  
    let seed \= \[42; 32\];  
    // Run this once, see what you get, and hard-code it  
    // Let's pretend it rolls a \[3, 5\]  
    let result \= roll\_with\_seed("2d6+5", seed).unwrap();  
      
    assert\_eq\!(result.total, 13); // 3 \+ 5 \+ 5  
    assert\_eq\!(result.dice\_rolls, vec\!\[3, 5\]);  
    assert\_eq\!(result.to\_string(), "\[3, 5\] \+ 5 \= 13");

    // Running it again with the same seed MUST give the same result  
    let result\_again \= roll\_with\_seed("2d6+5", seed).unwrap();  
    assert\_eq\!(result\_again.total, 13);  
}

### **⛔ PR 9: Implement Error Handling & Validation**

**Goal:** Implement the specific business logic errors from your requirements (0d6, 1d0, 1001d6).

**What to do:**

* After your parser runs (in both roll and roll\_with\_seed) and you have the DiceRequest struct, add validation logic.  
  let (remaining, request) \= your\_parser\_function(expression)?;  
  // ... check \`remaining\` is empty ...

  if request.quantity \> 1000 {  
      return Err(DiceError::QuantityLimitExceeded(request.quantity));  
  }  
  if request.quantity \<= 0 {  
      return Err(DiceError::InvalidQuantity(request.quantity));  
  }  
  if request.sides \<= 0 {  
      return Err(DiceError::InvalidDieSize(request.sides));  
  }

  // ... proceed with rolling ...

**What to test:** Add new unit tests for *each* error case.

// In src/lib.rs (in the test module)  
\#\[test\]  
fn test\_error\_quantity\_limit() {  
    assert\!(roll("1001d6").is\_err());  
}

\#\[test\]  
fn test\_error\_zero\_dice() {  
    assert\!(roll("0d20").is\_err());  
}

\#\[test\]  
fn test\_error\_zero\_sides() {  
    assert\!(roll("1d0").is\_err());  
}

### **📖 PR 10: Add examples/ and README.md**

**Goal:** Finish the project by adding the README.md and the examples/simple\_roll.rs file from your requirements. This is the final integration test.

**What to do:**

* Create a file: examples/simple\_roll.rs.  
* Copy/paste your "Example Usage" code into it. (You'll need to add use dice\_core::... at the top).  
* Create a README.md file.  
* Run cargo run \--example simple\_roll.

**What to test:** If the example runs and prints the expected "Success\!" and "Error as expected" messages, you are **done**.