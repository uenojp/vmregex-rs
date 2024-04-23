# Regex Engine: the Virtual Machine Aproach

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

## Example
```rust
$ cargo run --example grep 'system(d| )' /etc/passwd
systemd-network:x:100:102:systemd Network Management,,,:/run/systemd:/usr/sbin/nologin
systemd-resolve:x:101:103:systemd Resolver,,,:/run/systemd:/usr/sbin/nologin
systemd-timesync:x:103:106:systemd Time Synchronization,,,:/run/systemd:/usr/sbin/nologin
systemd-oom:x:108:116:systemd Userspace OOM Killer,,,:/run/systemd:/usr/sbin/nologin
sssd:x:118:125:SSSD system user,,,:/var/lib/sss:/usr/sbin/nologin
hplip:x:126:7:HPLIP system user,,,:/run/hplip:/bin/false
fwupd-refresh:x:128:137:fwupd-refresh user,,,:/run/systemd:/usr/sbin/nologin
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
