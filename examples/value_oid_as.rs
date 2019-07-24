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

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
fn main() {
    let oid: Vec<i32> = vec![libc::CTL_KERN, libc::KERN_CLOCKRATE];
    let val: Box<ClockInfo> = sysctl::Ctl { oid }.value_as().expect("could not get value");
    println!("{:?}", val);
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn main() {
    println!("This operation is not supported on Linux.");
}
