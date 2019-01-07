This crate provides a safe interface for reading and writing information to the kernel using the sysctl interface.

[![Current Version](https://img.shields.io/crates/v/sysctl.svg)](https://crates.io/crates/sysctl)


*FreeBSD and macOS are supported.*  
*Contributions for improvements and other platforms are welcome.*

### Documentation

Documentation is available here: https://johalun.github.io/sysctl-rs/

or, to generate documentation locally do:
```sh
$ git clone https://github.com/johalun/sysctl-rs && cd sysctl-rs
$ cargo doc --no-deps
$ firefox target/doc/sysctl/index.html
```

### Usage

Add to `Cargo.toml`

```toml
[dependencies]
sysctl = "0.3.0"
```

### macos

* Due to limitations in the sysctl(3) API, many of the methods of
  the `Ctl` take a mutable reference to `self` on macos.
* Sysctl descriptions are not available on macos.
* Some tests failures are ignored, as the respective sysctls do not
  exist on macos.

### Example

sysctl comes with several examples, see the examples folder:

* `value.rs`: shows how to get a sysctl value
* `value_as.rs`: parsing values as structures
* `value_oid_as.rs`: getting a sysctl from OID constants from the `libc` crate.
* `set_value.rs`: shows how to set a sysctl value
* `struct.rs`: reading data into a struct
* `temperature.rs`: parsing temperatures
* `iterate.rs`: showcases iteration over the sysctl tree

Run with:

```sh
$ cargo run --example iterate
```

Or in a separate crate:

```rust
extern crate sysctl;
use sysctl::{Ctl, CtlValue};

fn main() {
    let ctl = Ctl::new("kern.osrevision");
    println!("Description: {:?}", ctl.description().unwrap());

    let val_enum = ctl.value().unwrap();
    if let CtlValue::Int(val) = val_enum {
        println!("Value: {}", val);
    }
}
```

