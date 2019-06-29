#[cfg(any(target_os = "macos", target_os = "freebsd"))]
extern crate libc;
extern crate sysctl;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
use std::fmt;

// Copied from /usr/include/sys/time.h
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
#[derive(Debug)]
#[repr(C)]
struct ClockInfo {
    hz: libc::c_int,   /* clock frequency */
    tick: libc::c_int, /* micro-seconds per hz tick */
    spare: libc::c_int,
    stathz: libc::c_int, /* statistics clock frequency */
    profhz: libc::c_int, /* profiling clock frequency */
}

// Copied from /usr/include/sys/resource.h
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
#[repr(C)]
struct LoadAvg {
    ldavg: [u32; 3],
    fscale: u64,
}

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
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

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
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

#[cfg(not(any(target_os = "macos", target_os = "freebsd")))]
fn main() {}
