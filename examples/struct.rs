extern crate sysctl;
extern crate libc;

use libc::c_int;
use std::mem;

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

fn main() {

    let ctl = "kern.clockrate";
    println!("\nRead sysctl {} and parse result to struct ClockInfo", ctl);

    let d = sysctl::description(ctl).unwrap();
    println!("Description: {:?}", d);

    let val_enum = sysctl::value(ctl).unwrap();
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
