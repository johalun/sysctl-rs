// linux/ctl.rs

use super::funcs::{set_value, string_to_name, value};
use consts::*;
use ctl_error::SysctlError;
use ctl_info::CtlInfo;
use ctl_type::CtlType;
use ctl_value::CtlValue;
use std::str::FromStr;
use traits::Sysctl;

/// This struct represents a system control.
#[derive(Debug, Clone, PartialEq)]
pub struct Ctl {
    name: String,
}

impl FromStr for Ctl {
    type Err = SysctlError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(Ctl {
            name: string_to_name(name),
        })
    }
}

impl Ctl {
    pub fn path(&self) -> String {
        format!("/proc/sys/{}", self.name.replace(".", "/").replace("..", "."))
    }
}

impl Sysctl for Ctl {
    /// Construct a Ctl from the name.
    ///
    /// This is just a wrapper around `Ctl::from_str`.
    ///
    /// Returns a result containing the struct Ctl on success or a SysctlError
    /// on failure.
    ///
    /// # Example
    ///
    /// ```
    /// extern crate sysctl;
    /// use sysctl::Ctl;
    ///
    /// let ctl = Ctl::new("kernel.ostype");
    /// ```
    ///
    /// If the sysctl does not exist, `Err(SysctlError::NotFound)` is returned.
    /// ```
    /// extern crate sysctl;
    /// use sysctl::{Ctl, SysctlError};
    ///
    /// let ctl = Ctl::new("this.sysctl.does.not.exist");
    /// match ctl {
    ///     Err(SysctlError::NotFound(_)) => (),
    ///     Err(_) => panic!("Wrong error type returned"),
    ///     Ok(_) => panic!("Nonexistent sysctl seems to exist"),
    /// }
    /// ```
    fn new(name: &str) -> Result<Self, SysctlError> {
        Ctl::from_str(name)
    }

    /// Returns a result containing the sysctl name on success, or a
    /// SysctlError on failure.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate sysctl;
    /// # use sysctl::Ctl;
    /// let ctl = Ctl::new("kern.osrelease").expect("could not get sysctl");
    /// assert_eq!(ctl.name().expect("could not get name"), "kern.osrelease");
    /// ```
    fn name(&self) -> Result<String, SysctlError> {
        Ok(self.name.clone())
    }

    /// Returns a result containing the sysctl value type on success,
    /// or a Sysctl Error on failure.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate sysctl;
    /// # use sysctl::{Ctl, CtlType};
    /// let ctl = Ctl::new("kern.osrelease")
    ///     .expect("Could not get kern.osrelease sysctl");
    /// let value_type = ctl.value_type()
    ///         .expect("Could not get kern.osrelease value type");
    /// assert_eq!(value_type, CtlType::String);
    /// ```
    fn value_type(&self) -> Result<CtlType, SysctlError> {
        let md = std::fs::metadata(&self.path()).map_err(SysctlError::IoError)?;
        if md.is_dir() {
            Ok(CtlType::Node)
        } else {
            Ok(CtlType::String)
        }
    }

    /// Returns a result containing the sysctl description if success, or an
    /// Error on failure.
    ///
    /// # Example
    /// ```
    /// extern crate sysctl;
    /// use sysctl::Ctl;
    ///
    /// fn main() {
    ///     let osrevision = sysctl::Ctl::new("kern.osrevision")
    ///         .expect("could not get kern.osrevision sysctl");
    ///     println!("Description: {:?}", osrevision.description())
    /// }
    /// ```
    fn description(&self) -> Result<String, SysctlError> {
        Ok("[N/A]".to_owned())
    }

    /// Returns a result containing the sysctl value on success, or a
    /// SysctlError on failure.
    ///
    /// # Example
    /// ```
    /// extern crate sysctl;
    /// extern crate libc;
    ///
    /// fn main() {
    ///     let osrevision = sysctl::Ctl::new("kern.osrevision")
    ///         .expect("could not get kern.osrevisio sysctl");
    ///     println!("Value: {:?}", osrevision.value());
    /// }
    /// ```
    fn value(&self) -> Result<CtlValue, SysctlError> {
        value(&self.path())
    }

    /// Returns a result containing the sysctl value as String on
    /// success, or a SysctlError on failure.
    ///
    /// # Example
    /// ```
    /// extern crate sysctl;
    /// extern crate libc;
    ///
    /// fn main() {
    ///     let osrevision = sysctl::Ctl::new("kern.osrevision")
    ///         .expect("could not get kern.osrevisio sysctl");
    ///     println!("Value: {}", osrevision.value_string());
    /// }
    /// ```
    fn value_string(&self) -> Result<String, SysctlError> {
        self.value().map(|v| format!("{}", v))
    }

    /// A generic method that takes returns a result containing the sysctl
    /// value if success, or a SysctlError on failure.
    ///
    /// May only be called for sysctls of type Opaque or Struct.
    /// # Example
    /// ```
    /// extern crate sysctl;
    /// extern crate libc;
    ///
    /// use libc::c_int;
    ///
    /// #[derive(Debug)]
    /// #[repr(C)]
    /// struct ClockInfo {
    ///     hz: c_int, /* clock frequency */
    ///     tick: c_int, /* micro-seconds per hz tick */
    ///     spare: c_int,
    ///     stathz: c_int, /* statistics clock frequency */
    ///     profhz: c_int, /* profiling clock frequency */
    /// }
    ///
    /// fn main() {
    ///     let clockrate = sysctl::Ctl::new("kern.clockrate")
    ///         .expect("could not get clockrate sysctl");
    ///     println!("{:?}", clockrate.value_as::<ClockInfo>());
    /// }
    /// ```
    fn value_as<T>(&self) -> Result<Box<T>, SysctlError> {
        Err(SysctlError::NotSupported)
    }

    /// Sets the value of a sysctl.
    /// Fetches and returns the new value if successful, or returns a
    /// SysctlError on failure.
    /// # Example
    /// ```
    /// extern crate sysctl;
    /// use sysctl::Ctl;
    /// # extern crate libc;
    ///
    /// fn main() {
    /// # if unsafe { libc::getuid() } == 0 {
    ///     let usbdebug = Ctl::new("hw.usb.debug")
    ///         .expect("could not get hw.usb.debug control");
    /// #   let original = usbdebug.value()
    /// #       .expect("could not get value");
    ///     let set = usbdebug.set_value(sysctl::CtlValue::Int(1))
    ///         .expect("could not set value");
    ///     assert_eq!(set, sysctl::CtlValue::Int(1));
    ///     println!("hw.usb.debug: -> {:?}", set);
    /// #   usbdebug.set_value(original).unwrap();
    /// # } // getuid() == 0
    /// }
    fn set_value(&self, value: CtlValue) -> Result<CtlValue, SysctlError> {
        set_value(&self.path(), value)
    }

    /// Sets the value of a sysctl with input as string.
    /// Fetches and returns the new value if successful, or returns a
    /// SysctlError on failure.
    /// # Example
    /// ```ignore
    /// extern crate sysctl;
    /// use sysctl::*;
    ///
    /// fn main() {
    ///     let val = Ctl::new("net.ipv4.ip_forward")
    ///         .expect("could not get net.ipv4.ip_forward control");
    /// #   let original = val.value_string()
    /// #       .expect("could not get value");
    ///     let set = val.set_value_string("1");
    ///     println!("net.ipv4.ip_forward: -> {:?}", set);
    /// #   val.set_value_string(&original).unwrap();
    /// }
    fn set_value_string(&self, value: &str) -> Result<String, SysctlError> {
        self.set_value(CtlValue::String(value.to_owned()))?;
        self.value_string()
    }

    /// Get the flags for a sysctl.
    ///
    /// Returns a Result containing the flags on success,
    /// or a SysctlError on failure.
    ///
    /// # Example
    /// ```
    /// extern crate sysctl;
    /// use sysctl::{Ctl, CtlFlags};
    ///
    /// fn main() {
    ///     let osrev = Ctl::new("kern.osrevision")
    ///         .expect("could not get control");
    ///
    ///     let readable = osrev.flags()
    ///         .expect("could not get flags")
    ///         .contains(CtlFlags::RD);
    ///
    ///     assert!(readable);
    /// }
    /// ```
    fn flags(&self) -> Result<CtlFlags, SysctlError> {
        Ok(self.info()?.flags())
    }

    /// Returns a Result containing the control metadata for a sysctl.
    ///
    /// Returns a Result containing the CtlInfo struct on success,
    /// or a SysctlError on failure.
    ///
    /// # Example
    /// ```
    /// extern crate sysctl;
    /// use sysctl::{Ctl, CtlInfo};
    ///
    /// fn main() {
    ///     let osrev = Ctl::new("kern.osrevision")
    ///         .expect("could not get control");
    ///
    ///     let info = osrev.info()
    ///         .expect("could not get info");
    ///
    ///     // kern.osrevision is not a structure.
    ///     assert_eq!(info.struct_type(), None);
    /// }
    /// ```
    fn info(&self) -> Result<CtlInfo, SysctlError> {
        let md = std::fs::metadata(&self.path()).map_err(SysctlError::IoError)?;
        let mut flags = 0;
        if md.permissions().readonly() {
            flags |= CTLFLAG_RD;
        } else {
            flags |= CTLFLAG_RW;
        }
        let s = CtlInfo {
            ctl_type: CtlType::String,
            fmt: "".to_owned(),
            flags: flags,
        };
        Ok(s)
    }
}
