extern crate sysctl;

use std::io;

fn main() {
    #[cfg(any(target_os = "macos", target_os = "freebsd"))]
    const KEY: &str = "net.inet.ip.forwarding";
    #[cfg(any(target_os = "android", target_os = "linux"))]
    const KEY: &str = "/proc/sys/net/ipv4/ip_forward";

    #[cfg(any(target_os = "macos", target_os = "freebsd"))]
    let ret = sysctl::set_value(KEY, sysctl::CtlValue::Int(1))
        .map(|v| v == sysctl::CtlValue::Int(1))
        .map_err(|_| io::Error::last_os_error());

    #[cfg(any(target_os = "android", target_os = "linux"))]
    let ret = sysctl::set_value(KEY, sysctl::CtlValue::String("1\n".to_string()))
        .map(|v| v == sysctl::CtlValue::String("1\n".to_string()))
        .map_err(|_| io::Error::last_os_error());

    match ret {
        Ok(new_val) => {
            println!("The `{}` succcesfully set to: {}", KEY, new_val);
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
        }
    }
}
