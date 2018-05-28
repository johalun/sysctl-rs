extern crate libc;
extern crate sysctl;

#[cfg(not(target_os = "macos"))]
use libc::c_int;
#[cfg(not(target_os = "macos"))]
use std::mem;

// Copied from /usr/include/sys/time.h
#[derive(Debug)]
#[repr(C)]
#[cfg(not(target_os = "macos"))]
struct ClockInfo {
    hz: c_int,   /* clock frequency */
    tick: c_int, /* micro-seconds per hz tick */
    spare: c_int,
    stathz: c_int, /* statistics clock frequency */
    profhz: c_int, /* profiling clock frequency */
}
#[cfg(not(target_os = "macos"))]
fn main() {
    let ctl = sysctl::Ctl::new("kern.clockrate").expect("could not get sysctl: kern.clockrate");

    let name = ctl.name().expect("could not get sysctl name");
    println!("Read sysctl {} and parse result to struct ClockInfo", name);

    let d = ctl.description().expect("could not get sysctl description");
    println!("Description: {:?}", d);

    let val_enum = ctl.value().expect("could not get sysctl value");
    println!("ClockInfo raw data: {:?}", val_enum);

    if let sysctl::CtlValue::Struct(val) = val_enum {
        // Make sure we got correct data size
        assert_eq!(mem::size_of::<ClockInfo>(), val.len());
        let val_ptr: *const u8 = val.as_ptr();
        let struct_ptr: *const ClockInfo = val_ptr as *const ClockInfo;
        let struct_ref: &ClockInfo = unsafe { &*struct_ptr };
        println!("{:?}", struct_ref);
    }
}

#[cfg(target_os = "macos")]
fn main() {}
