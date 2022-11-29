#![allow(dead_code)]
#![allow(unused_imports)]

extern crate libc;
extern crate sysctl;

// Import the trait
use sysctl::Sysctl;

// Converted from definition in from /usr/include/sys/time.h
#[derive(Debug)]
#[repr(C)]
struct ClockInfo {
    hz: libc::c_int,   /* clock frequency */
    tick: libc::c_int, /* micro-seconds per hz tick */
    spare: libc::c_int,
    stathz: libc::c_int, /* statistics clock frequency */
    profhz: libc::c_int, /* profiling clock frequency */
}

#[cfg(any(target_os = "freebsd", target_os = "macos"))]
fn main() {
    let ctl = sysctl::Ctl::new("kern.clockrate").expect("could not get sysctl: kern.clockrate");

    let name = ctl.name().expect("could not get sysctl name");
    println!("Read sysctl {} and parse result to struct ClockInfo", name);

    let d = ctl.description().expect("could not get sysctl description");
    println!("Description: {:?}", d);

    if let Ok(s) = ctl.value_as::<ClockInfo>() {
        println!("{:?}", s);
    }
}

#[cfg(not(any(target_os = "freebsd", target_os = "macos")))]
fn main() {
    println!("This operation is only supported on FreeBSD and macOS.");
}
