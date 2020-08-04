#![allow(dead_code)]
#![allow(unused_imports)]

extern crate sysctl;

// Import the trait
use sysctl::Sysctl;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
const CTLNAMES: &[&str] = &["kern.ostype"];

#[cfg(any(target_os = "linux", target_os = "android"))]
const CTLNAMES: &[&str] = &["kernel.ostype", "kernel/ostype", "/proc/sys/kernel/ostype"];

fn print_ctl(ctlname: &str) -> Result<(), sysctl::SysctlError> {
    println!("Reading '{}'", ctlname);
    let ctl = sysctl::Ctl::new(ctlname)?;
    let desc = ctl.description()?;
    println!("Description: {}", desc);
    let val = ctl.value()?;
    println!("Value: {}", val);
    Ok(())
}

fn main() {
    for ctlname in CTLNAMES {
        print_ctl(ctlname).unwrap_or_else(|e: sysctl::SysctlError| {
            eprintln!("Error: {:?}", e);
        });
    }
}
