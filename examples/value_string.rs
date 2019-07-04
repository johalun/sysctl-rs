extern crate sysctl;

// Import the trait
use sysctl::Sysctl;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
const CTLNAMES: &[&str] = &["kern.osrevision"];

// On Linux all sysctl are String so doesn't really make sense here...
#[cfg(any(target_os = "linux", target_os = "android"))]
const CTLNAMES: &[&str] = &["kernel.overflowuid"];

fn print_ctl(ctlname: &str) -> Result<(), sysctl::SysctlError> {
    println!("Reading '{}'", ctlname);
    let ctl = try!(sysctl::Ctl::new(ctlname));
    let description = try!(ctl.description());
    println!("Description: {}", description);
    let val_string = try!(ctl.value_string());
    println!("Value: {}", val_string);
    Ok(())
}

fn main() {
    for ctlname in CTLNAMES {
        print_ctl(ctlname).unwrap_or_else(|e: sysctl::SysctlError| {
            eprintln!("Error: {:?}", e);
        });
    }
}
