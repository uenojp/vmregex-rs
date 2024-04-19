# Regex Engine: the Virtual Machine Aproach

## Example
```rust
use vmregex::Regex;

let re = Regex::new("Hel+o (Wo*rld|R.+st)!?").unwrap();
assert!(re.is_match("Hello World!").unwrap());
assert!(re.is_match("Helllllo Wrld").unwrap());
assert!(re.is_match("Hello Rust").unwrap());
assert!(re.is_match("Helllllo Rxxxxxxst").unwrap());
assert!(!re.is_match("Heo World!").unwrap());
assert!(!re.is_match("Hello Rst!").unwrap());
```

## Operation
- Concatenation
- Alternation
- Question
- Star
- Plus
- Dot

## Reference
- https://github.com/ytakano/rust_zero
- https://swtch.com/~rsc/regexp/regexp2.html
