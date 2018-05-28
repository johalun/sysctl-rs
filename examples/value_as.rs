extern crate libc;
extern crate sysctl;

use libc::c_int;
use std::fmt;

// Copied from /usr/include/sys/time.h
#[derive(Debug)]
#[repr(C)]
struct ClockInfo {
    hz: c_int,   /* clock frequency */
    tick: c_int, /* micro-seconds per hz tick */
    spare: c_int,
    stathz: c_int, /* statistics clock frequency */
    profhz: c_int, /* profiling clock frequency */
}

// Copied from /usr/include/sys/resource.h
#[repr(C)]
struct LoadAvg {
    ldavg: [u32; 3],
    fscale: u64,
}
impl fmt::Debug for LoadAvg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.fscale as f32;
        write!(
            f,
            "LoadAvg {{ {:.2} {:.2} {:.2} }}",
            self.ldavg[0] as f32 / s,
            self.ldavg[1] as f32 / s,
            self.ldavg[2] as f32 / s
        )
    }
}

fn main() {
    // Generic type to pass to function will be inferred if not specified on RHS
    println!("\nRead sysctl kern.clockrate as struct directly");
    let val: Box<ClockInfo> = sysctl::Ctl::new("kern.clockrate")
        .expect("could not get sysctl: kern.clockrate")
        .value_as()
        .expect("could not read sysctl as struct");
    println!("{:?}", val);

    // Pass type LoadAvg to generic function
    println!("\nRead sysctl vm.loadavg as struct directly");
    let val = sysctl::Ctl::new("vm.loadavg")
        .expect("could not get sysctl: vm.loadavg")
        .value_as::<LoadAvg>()
        .expect("could not read sysctl as LoadAvg");
    println!("{:?}", val);
}
