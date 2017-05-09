This crate provides a safe interface for reading and writing information to the kernel using the `sysctl` interface.

### Documentation

https://docs.rs/sysctl

### Usage

Add to `Cargo.toml`

```toml
[dependencies]
sysctl = "0.1.0"
```

### Example

```rust
extern crate sysctl;

fn main() {
    let ctl = "kern.osrevision";
    let d: String = sysctl::description(ctl).unwrap();
    println!("Description: {:?}", d);

    let val_enum = sysctl::value(ctl).unwrap();
    if let sysctl::CtlValue::Int(val) = val_enum {
        println!("Value: {}", val);
    }
}
```




