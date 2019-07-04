// unix/ctl.rs

use super::funcs::{
    name2oid, next_oid, oid2description, oid2name, oidfmt, set_oid_value, value_oid, value_oid_as,
};
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
    pub oid: Vec<libc::c_int>,
}

impl std::str::FromStr for Ctl {
    type Err = SysctlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let oid = name2oid(s)?;

        Ok(Ctl { oid: oid })
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
    /// let ctl = Ctl::new("kern.osrelease");
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
        oid2name(&self.oid)
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
        let info = oidfmt(&self.oid)?;
        Ok(info.ctl_type)
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
    #[cfg(not(target_os = "macos"))]
    fn description(&self) -> Result<String, SysctlError> {
        oid2description(&self.oid)
    }

    #[cfg(target_os = "macos")]
    fn description(&self) -> Result<String, SysctlError> {
        Ok("[N/A]".to_string())
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
    #[cfg(not(target_os = "macos"))]
    fn value(&self) -> Result<CtlValue, SysctlError> {
        value_oid(&self.oid)
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
    #[cfg(target_os = "macos")]
    fn value(&self) -> Result<CtlValue, SysctlError> {
        let mut oid = self.oid.clone();
        value_oid(&mut oid)
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
    /// use libc::libc::c_int;
    ///
    /// #[derive(Debug)]
    /// #[repr(C)]
    /// struct ClockInfo {
    ///     hz: libc::c_int, /* clock frequency */
    ///     tick: libc::c_int, /* micro-seconds per hz tick */
    ///     spare: libc::c_int,
    ///     stathz: libc::c_int, /* statistics clock frequency */
    ///     profhz: libc::c_int, /* profiling clock frequency */
    /// }
    ///
    /// fn main() {
    ///     let clockrate = sysctl::Ctl::new("kern.clockrate")
    ///         .expect("could not get clockrate sysctl");
    ///     println!("{:?}", clockrate.value_as::<ClockInfo>());
    /// }
    /// ```
    #[cfg(not(target_os = "macos"))]
    fn value_as<T>(&self) -> Result<Box<T>, SysctlError> {
        value_oid_as::<T>(&self.oid)
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
    /// use libc::libc::c_int;
    ///
    /// #[derive(Debug)]
    /// #[repr(C)]
    /// struct ClockInfo {
    ///     hz: libc::c_int, /* clock frequency */
    ///     tick: libc::c_int, /* micro-seconds per hz tick */
    ///     spare: libc::c_int,
    ///     stathz: libc::c_int, /* statistics clock frequency */
    ///     profhz: libc::c_int, /* profiling clock frequency */
    /// }
    ///
    /// fn main() {
    ///     let clockrate = sysctl::Ctl::new("kern.clockrate")
    ///         .expect("could not get clockrate sysctl");
    ///     println!("{:?}", clockrate.value_as::<ClockInfo>());
    /// }
    /// ```
    #[cfg(target_os = "macos")]
    fn value_as<T>(&self) -> Result<Box<T>, SysctlError> {
        let mut oid = self.oid.clone();
        value_oid_as::<T>(&mut oid)
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
    #[cfg(not(target_os = "macos"))]
    fn set_value(&self, value: CtlValue) -> Result<CtlValue, SysctlError> {
        set_oid_value(&self.oid, value)
    }

    /// Sets the value of a sysctl.
    /// Fetches and returns the new value if successful, or returns a
    /// SysctlError on failure.
    /// # Example
    /// ```ignore
    /// extern crate sysctl;
    /// use sysctl::Ctl;
    ///
    /// fn main() {
    ///     let usbdebug = Ctl::new("hw.usb.debug")
    ///         .expect("could not get hw.usb.debug control");
    /// #   let original = usbdebug.value()
    /// #       .expect("could not get value");
    ///     let set = usbdebug.set_value(sysctl::CtlValue::Int(1));
    ///     println!("hw.usb.debug: -> {:?}", set);
    /// #   usbdebug.set_value(original).unwrap();
    /// }
    #[cfg(target_os = "macos")]
    fn set_value(&self, value: CtlValue) -> Result<CtlValue, SysctlError> {
        let mut oid = self.oid.clone();
        set_oid_value(&mut oid, value)
    }

    /// Sets the value of a sysctl with input as string.
    /// Fetches and returns the new value if successful, or returns a
    /// SysctlError on failure.
    /// # Example
    /// ```ignore
    /// extern crate sysctl;
    /// use sysctl::Ctl;
    ///
    /// fn main() {
    ///     let usbdebug = Ctl::new("hw.usb.debug")
    ///         .expect("could not get hw.usb.debug control");
    /// #   let original = usbdebug.value_string()
    /// #       .expect("could not get value");
    ///     let set = usbdebug.set_value_string("1");
    ///     println!("hw.usb.debug: -> {:?}", set);
    /// #   usbdebug.set_value_string(&original).unwrap();
    /// }
    fn set_value_string(&self, value: &str) -> Result<String, SysctlError> {
        let ctl_type = try!(self.value_type());
        let _ = match ctl_type {
            CtlType::String => set_oid_value(&self.oid, CtlValue::String(value.to_owned())),
            CtlType::Int => set_oid_value(
                &self.oid,
                CtlValue::Int(value.parse::<i32>().map_err(|_| SysctlError::ParseError)?),
            ),
            _ => Err(SysctlError::MissingImplementation),
        }?;
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
        Ok(oidfmt(&self.oid)?)
    }
}
