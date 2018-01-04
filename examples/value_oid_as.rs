extern crate sysctl;
extern crate libc;

use libc::c_int;

// Copied from /usr/include/sys/time.h
#[derive(Debug)]
#[repr(C)]
struct ClockInfo {
    hz: c_int, /* clock frequency */
    tick: c_int, /* micro-seconds per hz tick */
    spare: c_int,
    stathz: c_int, /* statistics clock frequency */
    profhz: c_int, /* profiling clock frequency */
}
#[cfg(not(target_os = "macos"))]
fn main() {
    let oid: Vec<i32> = vec![libc::CTL_KERN, libc::KERN_CLOCKRATE];
    let val: Box<ClockInfo> = sysctl::value_oid_as(&oid).unwrap();
    println!("{:?}", val);
}

#[cfg(target_os = "macos")]
fn main() {
    let mut oid: Vec<i32> = vec![libc::CTL_KERN, libc::KERN_CLOCKRATE];
    let val: Box<ClockInfo> = sysctl::value_oid_as(&mut oid).unwrap();
    println!("{:?}", val);
}