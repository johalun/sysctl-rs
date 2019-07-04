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

// Converted from definition in /usr/include/sys/resource.h
#[repr(C)]
struct LoadAvg {
    ldavg: [u32; 3],
    fscale: u64,
}
impl std::fmt::Debug for LoadAvg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
fn main() {
    // Generic type to pass to function will be inferred if not specified on RHS
    println!("Read sysctl kern.clockrate as struct directly");
    let val: Box<ClockInfo> = sysctl::Ctl::new("kern.clockrate")
        .expect("could not get sysctl: kern.clockrate")
        .value_as()
        .expect("could not read sysctl as struct");
    println!("{:?}", val);
    println!();

    // Pass type LoadAvg to generic function
    println!("Read sysctl vm.loadavg as struct directly");
    let val = sysctl::Ctl::new("vm.loadavg")
        .expect("could not get sysctl: vm.loadavg")
        .value_as::<LoadAvg>()
        .expect("could not read sysctl as LoadAvg");
    println!("{:?}", val);
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn main() {
    println!("This operation is not supported on Linux.");
}
