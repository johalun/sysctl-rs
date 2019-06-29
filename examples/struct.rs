#[cfg(any(target_os = "macos", target_os = "freebsd"))]
extern crate libc;
extern crate sysctl;

// Copied from /usr/include/sys/time.h
#[derive(Debug)]
#[repr(C)]
#[cfg(target_os = "freebsd")]
struct ClockInfo {
    hz: libc::c_int,   /* clock frequency */
    tick: libc::c_int, /* micro-seconds per hz tick */
    spare: libc::c_int,
    stathz: libc::c_int, /* statistics clock frequency */
    profhz: libc::c_int, /* profiling clock frequency */
}

#[cfg(target_os = "freebsd")]
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
        assert_eq!(std::mem::size_of::<ClockInfo>(), val.len());
        let val_ptr: *const u8 = val.as_ptr();
        let struct_ptr: *const ClockInfo = val_ptr as *const ClockInfo;
        let struct_ref: &ClockInfo = unsafe { &*struct_ptr };
        println!("{:?}", struct_ref);
    }
}

#[cfg(not(target_os = "freebsd"))]
fn main() {}
