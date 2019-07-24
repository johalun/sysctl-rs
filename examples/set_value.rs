#![allow(dead_code)]
#![allow(unused_imports)]

extern crate libc;
extern crate sysctl;

// Import the trait
use sysctl::Sysctl;

#[cfg(target_os = "freebsd")]
const CTLNAME: &str = "net.inet.ip.forwarding";

#[cfg(target_os = "macos")]
const CTLNAME: &str = "net.inet.ip.forwarding";

#[cfg(any(target_os = "linux", target_os = "android"))]
const CTLNAME: &str = "net.ipv4.ip_forward";

fn main() {
    assert_eq!(
        unsafe { libc::geteuid() },
        0,
        "This example must be run as root"
    );

    let ctl = sysctl::Ctl::new(CTLNAME).expect(&format!("could not get sysctl '{}'", CTLNAME));

    let name = ctl.name().expect("could not get sysctl name");
    println!("\nFlipping value of sysctl '{}'", name);

    let old_value = ctl.value_string().expect("could not get sysctl value");
    println!("Current value is '{}'", old_value);

    let target_value = match old_value.as_ref() {
        "0" => "1",
        _ => "0",
    };

    println!("Setting value to '{}'...", target_value);
    let new_value = ctl.set_value_string(target_value).unwrap_or_else(|e| {
        panic!("Could not set value. Error: {:?}", e);
    });
    assert_eq!(new_value, target_value, "could not set value");
    println!("OK. Now restoring old value '{}'...", old_value);

    let ret = ctl
        .set_value_string(&old_value)
        .expect("could not restore old value");
    println!("OK. Value restored to {}.", ret);
}
