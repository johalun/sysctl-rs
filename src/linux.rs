use crate::{CtlValue, SysctlError};

use std::fs::OpenOptions;
use std::io::{self, Read, Write};

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
#[cfg(target_env = "musl")]
pub fn value(name: &str) -> Result<CtlValue, SysctlError> {
    let file_res = if name.starts_with("/proc/sys") {
        OpenOptions::new().read(true).write(false).open(&name)
    } else {
        let name = format!("/proc/sys/{}", name.replace(".", "/"));
        OpenOptions::new().read(true).write(false).open(&name)
    };

    file_res
        .map(|mut file| {
            let mut v = String::new();
            file.read_to_string(&mut v)?;
            Ok(CtlValue::String(v))
        })
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
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
#[cfg(target_env = "musl")]
pub fn set_value(name: &str, v: CtlValue) -> Result<CtlValue, SysctlError> {
    let file_res = if name.starts_with("/proc/sys") {
        OpenOptions::new().read(false).write(true).open(&name)
    } else {
        let name = format!("/proc/sys/{}", name.replace(".", "/"));
        OpenOptions::new().read(false).write(true).open(&name)
    };

    file_res
        .map(|mut file| match v {
            CtlValue::String(v) => {
                file.write_all(&v.as_bytes())?;
                value(name)
            }
            _ => Err(io::Error::from(io::ErrorKind::InvalidData).into()),
        })
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                SysctlError::NotFound(name.into())
            } else {
                e.into()
            }
        })?
}
