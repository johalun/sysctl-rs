// linux/funcs.rs

use super::ctl::Ctl;
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use consts::*;
use ctl_error::*;
use ctl_info::*;
use ctl_type::*;
use ctl_value::*;
use traits::Sysctl;

use std::io::{Read, Write};

pub fn fix_path(name: &str) -> String {
    if name.starts_with("/proc/sys") {
        name.to_owned()
    } else if name.contains("/") {
        format!("/proc/sys/{}", name)
    } else {
        format!("/proc/sys/{}", name.replace(".", "/"))
    }
}

/// Takes the name of the OID as argument and returns
/// a result containing the sysctl value if success,
/// or a SysctlError on failure
///
/// # Example
/// ```
/// extern crate sysctl;
///
/// fn main() {
///     let val = sysctl::value("/proc/sys/net/ipv4/ip_forward")
///                  .map(|v| v == sysctl::CtlValue::String("1\n".to_string()))
///                  .map_err(|_| std::io::Error::last_os_error())
///                  .unwrap();
///     println!("Value: {}", val);
/// }
/// ```
pub fn value(name: &str) -> Result<CtlValue, SysctlError> {
    let name = fix_path(name);
    let file_res = std::fs::OpenOptions::new()
        .read(true)
        .write(false)
        .open(&name);

    file_res
        .map(|mut file| {
            let mut v = String::new();
            file.read_to_string(&mut v)?;
            Ok(CtlValue::String(v.trim().to_owned()))
        })
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SysctlError::NotFound(name.into())
            } else {
                e.into()
            }
        })?
}

/// Sets the value of a sysctl.
/// Fetches and returns the new value if successful, or a SysctlError
/// on failure
///
/// # Example
/// ```
/// extern crate sysctl;
///
/// fn main() {
///     let val = sysctl::CtlValue::String("1\n".to_string());
///     let ret = sysctl::set_value("/proc/sys/net/ipv4/ip_forward", val);
///     println!("set value ret: {:?}", ret);
/// }
/// ```
pub fn set_value(name: &str, v: CtlValue) -> Result<CtlValue, SysctlError> {
    let name = fix_path(name);
    let file_res = std::fs::OpenOptions::new()
        .read(false)
        .write(true)
        .open(&name);

    file_res
        .map(|mut file| match v {
            CtlValue::String(v) => {
                file.write_all(&v.as_bytes())?;
                value(&name)
            }
            _ => Err(std::io::Error::from(std::io::ErrorKind::InvalidData).into()),
        })
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SysctlError::NotFound(name.into())
            } else {
                e.into()
            }
        })?
}

pub fn next_ctl(ctl: &Ctl) -> Result<Ctl, SysctlError> {
    Ctl::new("")
}
