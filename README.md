This crate provides a safe interface for reading and writing information to the kernel using the sysctl interface.

[![Current Version](https://img.shields.io/crates/v/sysctl.svg)](https://crates.io/crates/sysctl)


*Currently only developed and tested on FreeBSD.*  
*Contributions for improvements and other platforms are welcome.*

### Documentation

Since the crate only builds on FreeBSD documentation is not available on https://docs.rs/sysctl

Available here: https://johalun.github.io/sysctl-rs/sysctl/index.html

or, to generate documentation locally do:
```sh
$ git clone https://github.com/johalun/sysctl-rs && cd sysctl-rs
$ cargo doc
$ firefox target/doc/sysctl/index.html
```

### Usage

Add to `Cargo.toml`

```toml
[dependencies]
sysctl = "0.1.2"
```

### Example

sysctl comes with several examples, see the examples folder.

Run with:

```sh
$ cargo run --example value
```

Or in a separate crate:


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




