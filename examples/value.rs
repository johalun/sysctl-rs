extern crate sysctl;

use sysctl::{Ctl, CtlValue};

#[cfg(not(target_os = "macos"))]
fn main() {
    let ctl = Ctl::new("kern.osrevision").expect("could not get sysctl");

    let name = ctl.name().expect("could not get name");

    println!("\nRead sysctl {}", name);

    let description = ctl.description().expect("could not get description");

    println!("Description: {:?}", description);

    let val_enum = ctl.value().expect("could not get sysctl value");

    if let CtlValue::Int(val) = val_enum {
        println!("Value: {}", val);
    }
}

#[cfg(target_os = "macos")]
fn main() {
    // on macos the `name` and `newp` parameters of the sysctl(3)
    // syscall API are not marked `const`. This means the sysctl
    // structure has to be mutable.
    let ctl = Ctl::new("kern.osrevision").expect("could not get sysctl");

    let name = ctl.name().expect("could not get name");

    println!("\nRead sysctl {}", name);

    // sysctl descriptions are not available on macos.

    let val_enum = ctl.value().expect("could not get sysctl value");

    if let CtlValue::Int(val) = val_enum {
        println!("Value: {}", val);
    }
}
