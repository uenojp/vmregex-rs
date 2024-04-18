# Regex Engine: the Virtual Machine Aproach

## Example
```rust
use vmregex::Regex;

let re = Regex::new("a(b|c)*d+").unwrap();
assert!(re.is_match("ad").unwrap());
assert!(re.is_match("abbbbd").unwrap());
assert!(re.is_match("abcbcbcd").unwrap());
assert!(re.is_match("add").unwrap());
assert!(!re.is_match("aaa").unwrap());
```

## Operation
- Concatenation
- Alternation
- Question
- Star
- Plus

## Reference
- https://github.com/ytakano/rust_zero
- https://swtch.com/~rsc/regexp/regexp2.html
