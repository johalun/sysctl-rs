//! A simplified interface to the `sysctl` system call.
//!
//! Currently built for and only tested on FreeBSD.
//!
//! # Example: Get description and value
//! ```
//! extern crate sysctl;
//! #[cfg(not(target_os = "macos"))]
//! fn main() {
//!     let ctl = sysctl::Ctl::new("kern.osrevision")
//!         .expect("could not get sysctl");
//!
//!     let d = ctl.description()
//!         .expect("could not get description");
//!
//!     println!("Description: {:?}", d);
//!
//!     let val_enum = ctl.value()
//!         .expect("could not get value");
//!
//!     if let sysctl::CtlValue::Int(val) = val_enum {
//!         println!("Value: {}", val);
//!     }
//! }
//!
//! #[cfg(target_os = "macos")]
//! fn main() {
//!     let mut ctl = sysctl::Ctl::new("kern.osrevision")
//!         .expect("could not get sysctl");
//!
//!     // description is not available on macos
//!
//!     let val_enum = ctl.value()
//!         .expect("could not get value");
//!
//!     if let sysctl::CtlValue::Int(val) = val_enum {
//!         println!("Value: {}", val);
//!     }
//! }
//! ```
//! # Example: Get value as struct
//! ```
//! extern crate sysctl;
//! extern crate libc;
//!
//! use libc::c_int;
//!
//! #[derive(Debug)]
//! #[repr(C)]
//! struct ClockInfo {
//!     hz: c_int, /* clock frequency */
//!     tick: c_int, /* micro-seconds per hz tick */
//!     spare: c_int,
//!     stathz: c_int, /* statistics clock frequency */
//!     profhz: c_int, /* profiling clock frequency */
//! }
//!
//! fn main() {
//!     println!("{:?}", sysctl::value_as::<ClockInfo>("kern.clockrate"));
//! }
//! ```

#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate libc;

#[macro_use]
extern crate failure;

use libc::sysctl;
use libc::BUFSIZ;
use libc::{c_int, c_uchar, c_uint, c_void};

use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use std::cmp;
use std::convert;
#[cfg(not(target_os = "macos"))]
use std::f32;
use std::io;
use std::mem;
use std::ptr;
use std::str;
use std::str::FromStr;

// CTL* constants belong to libc crate but have not been added there yet.
// They will be removed from here once in the libc crate.
pub const CTL_MAXNAME: c_uint = 24;

pub const CTLTYPE: c_uint = 0xf; /* mask for the type */

pub const CTLTYPE_NODE: c_uint = 1;
pub const CTLTYPE_INT: c_uint = 2;
pub const CTLTYPE_STRING: c_uint = 3;
pub const CTLTYPE_S64: c_uint = 4;
pub const CTLTYPE_OPAQUE: c_uint = 5;
pub const CTLTYPE_STRUCT: c_uint = 5;
pub const CTLTYPE_UINT: c_uint = 6;
pub const CTLTYPE_LONG: c_uint = 7;
pub const CTLTYPE_ULONG: c_uint = 8;
pub const CTLTYPE_U64: c_uint = 9;
pub const CTLTYPE_U8: c_uint = 10;
pub const CTLTYPE_U16: c_uint = 11;
pub const CTLTYPE_S8: c_uint = 12;
pub const CTLTYPE_S16: c_uint = 13;
pub const CTLTYPE_S32: c_uint = 14;
pub const CTLTYPE_U32: c_uint = 15;

pub const CTLFLAG_RD: c_uint = 0x80000000;
pub const CTLFLAG_WR: c_uint = 0x40000000;
pub const CTLFLAG_RW: c_uint = 0x80000000 | 0x40000000;
pub const CTLFLAG_DORMANT: c_uint = 0x20000000;
pub const CTLFLAG_ANYBODY: c_uint = 0x10000000;
pub const CTLFLAG_SECURE: c_uint = 0x08000000;
pub const CTLFLAG_PRISON: c_uint = 0x04000000;
pub const CTLFLAG_DYN: c_uint = 0x02000000;
pub const CTLFLAG_SKIP: c_uint = 0x01000000;
pub const CTLFLAG_TUN: c_uint = 0x00080000;
pub const CTLFLAG_RDTUN: c_uint = CTLFLAG_RD | CTLFLAG_TUN;
pub const CTLFLAG_RWTUN: c_uint = CTLFLAG_RW | CTLFLAG_TUN;
pub const CTLFLAG_MPSAFE: c_uint = 0x00040000;
pub const CTLFLAG_VNET: c_uint = 0x00020000;
pub const CTLFLAG_DYING: c_uint = 0x00010000;
pub const CTLFLAG_CAPRD: c_uint = 0x00008000;
pub const CTLFLAG_CAPWR: c_uint = 0x00004000;
pub const CTLFLAG_STATS: c_uint = 0x00002000;
pub const CTLFLAG_NOFETCH: c_uint = 0x00001000;
pub const CTLFLAG_CAPRW: c_uint = CTLFLAG_CAPRD | CTLFLAG_CAPWR;
pub const CTLFLAG_SECURE1: c_uint = 134217728;
pub const CTLFLAG_SECURE2: c_uint = 135266304;
pub const CTLFLAG_SECURE3: c_uint = 136314880;

pub const CTLMASK_SECURE: c_uint = 15728640;
pub const CTLSHIFT_SECURE: c_uint = 20;

/// Represents control flags of a sysctl
bitflags! {
    pub struct CtlFlags : c_uint {
        /// Allow reads of variable
        const RD = CTLFLAG_RD;

        /// Allow writes to the variable
        const WR = CTLFLAG_WR;

        const RW = Self::RD.bits | Self::WR.bits;

        /// This sysctl is not active yet
        const DORMANT = CTLFLAG_DORMANT;

        /// All users can set this var
        const ANYBODY = CTLFLAG_ANYBODY;

        /// Permit set only if securelevel<=0
        const SECURE = CTLFLAG_SECURE;

        /// Prisoned roots can fiddle
        const PRISON = CTLFLAG_PRISON;

        /// Dynamic oid - can be freed
        const DYN = CTLFLAG_DYN;

        /// Skip this sysctl when listing
        const SKIP = CTLFLAG_DORMANT;

        /// Secure level
        const SECURE_MASK = 0x00F00000;

        /// Default value is loaded from getenv()
        const TUN = CTLFLAG_TUN;

        /// Readable tunable
        const RDTUN = Self::RD.bits | Self::TUN.bits;

        /// Readable and writeable tunable
        const RWTUN = Self::RW.bits | Self::TUN.bits;

        /// Handler is MP safe
        const MPSAFE = CTLFLAG_MPSAFE;

        /// Prisons with vnet can fiddle
        const VNET = CTLFLAG_VNET;

        /// Oid is being removed
        const DYING = CTLFLAG_DYING;

        /// Can be read in capability mode
        const CAPRD = CTLFLAG_CAPRD;

        /// Can be written in capability mode
        const CAPWR = CTLFLAG_CAPWR;

        /// Statistics; not a tuneable
        const STATS = CTLFLAG_STATS;

        /// Don't fetch tunable from getenv()
        const NOFETCH = CTLFLAG_NOFETCH;

        /// Can be read and written in capability mode
        const CAPRW = Self::CAPRD.bits | Self::CAPWR.bits;
    }
}

/// An Enum that represents a sysctl's type information.
///
/// # Example
///
/// ```
/// extern crate sysctl;
///
/// let val_enum = &sysctl::value("kern.osrevision")
///     .expect("could not get kern.osrevision sysctl");
///
/// let val_type: sysctl::CtlType = val_enum.into();
///
/// assert_eq!(val_type, sysctl::CtlType::Int);
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum CtlType {
    Node = 1,
    Int = 2,
    String = 3,
    S64 = 4,
    Struct = 5,
    Uint = 6,
    Long = 7,
    Ulong = 8,
    U64 = 9,
    U8 = 10,
    U16 = 11,
    S8 = 12,
    S16 = 13,
    S32 = 14,
    U32 = 15,
    // Added custom types below
    None = 0,
    #[cfg(not(target_os = "macos"))]
    Temperature = 16,
}
impl convert::From<u32> for CtlType {
    fn from(t: u32) -> Self {
        assert!(t <= 16);
        unsafe { mem::transmute(t) }
    }
}
impl<'a> convert::From<&'a CtlValue> for CtlType {
    fn from(t: &'a CtlValue) -> Self {
        match t {
            &CtlValue::None => CtlType::None,
            &CtlValue::Node(_) => CtlType::Node,
            &CtlValue::Int(_) => CtlType::Int,
            &CtlValue::String(_) => CtlType::String,
            &CtlValue::S64(_) => CtlType::S64,
            &CtlValue::Struct(_) => CtlType::Struct,
            &CtlValue::Uint(_) => CtlType::Uint,
            &CtlValue::Long(_) => CtlType::Long,
            &CtlValue::Ulong(_) => CtlType::Ulong,
            &CtlValue::U64(_) => CtlType::U64,
            &CtlValue::U8(_) => CtlType::U8,
            &CtlValue::U16(_) => CtlType::U16,
            &CtlValue::S8(_) => CtlType::S8,
            &CtlValue::S16(_) => CtlType::S16,
            &CtlValue::S32(_) => CtlType::S32,
            &CtlValue::U32(_) => CtlType::U32,
            #[cfg(not(target_os = "macos"))]
            &CtlValue::Temperature(_) => CtlType::Temperature,
        }
    }
}

impl CtlType {
    fn min_type_size(self: &Self) -> usize {
        match self {
            &CtlType::None => 0,
            &CtlType::Node => 0,
            &CtlType::Int => mem::size_of::<libc::c_int>(),
            &CtlType::String => 0,
            &CtlType::S64 => mem::size_of::<i64>(),
            &CtlType::Struct => 0,
            &CtlType::Uint => mem::size_of::<libc::c_uint>(),
            &CtlType::Long => mem::size_of::<libc::c_long>(),
            &CtlType::Ulong => mem::size_of::<libc::c_ulong>(),
            &CtlType::U64 => mem::size_of::<u64>(),
            &CtlType::U8 => mem::size_of::<u8>(),
            &CtlType::U16 => mem::size_of::<u16>(),
            &CtlType::S8 => mem::size_of::<i8>(),
            &CtlType::S16 => mem::size_of::<i16>(),
            &CtlType::S32 => mem::size_of::<i32>(),
            &CtlType::U32 => mem::size_of::<u32>(),
            // Added custom types below
            #[cfg(not(target_os = "macos"))]
            &CtlType::Temperature => 0,
        }
    }
}

/// An Enum that holds all values returned by sysctl calls.
/// Extract inner value with `if let` or `match`.
///
/// # Example
///
/// ```ignore
/// let val_enum = sysctl::value("kern.osrevision");
///
/// if let sysctl::CtlValue::Int(val) = val_enum {
///     println!("Value: {}", val);
/// }
/// ```
#[derive(Debug, PartialEq, PartialOrd)]
pub enum CtlValue {
    None,
    Node(Vec<u8>),
    Int(i32),
    String(String),
    S64(u64),
    Struct(Vec<u8>),
    Uint(u32),
    Long(i64),
    Ulong(u64),
    U64(u64),
    U8(u8),
    U16(u16),
    S8(i8),
    S16(i16),
    S32(i32),
    U32(u32),
    #[cfg(not(target_os = "macos"))]
    Temperature(Temperature),
}

#[derive(Debug, PartialEq)]
/// A structure representing control metadata
pub struct CtlInfo {
    /// The control type.
    pub ctl_type: CtlType,

    /// A string which specifies the format of the OID in
    /// a symbolic way.
    ///
    /// This format is used as a hint by sysctl(8) to
    /// apply proper data formatting for display purposes.
    ///
    /// Formats defined in sysctl(9):
    /// * `N`       node
    /// * `A`       char *
    /// * `I`       int
    /// * `IK[n]`   temperature in Kelvin, multiplied by an optional single
    ///    digit power of ten scaling factor: 1 (default) gives deciKelvin,
    ///    0 gives Kelvin, 3 gives milliKelvin
    /// * `IU`      unsigned int
    /// * `L`       long
    /// * `LU`      unsigned long
    /// * `Q`       quad_t
    /// * `QU`      u_quad_t
    /// * `S,TYPE`  struct TYPE structures
    pub fmt: String,

    flags: u32,
}

impl CtlInfo {
    /// Return the flags for this sysctl.
    pub fn flags(&self) -> CtlFlags {
        CtlFlags::from_bits_truncate(self.flags)
    }

    /// Is this sysctl a temperature?
    #[cfg(not(target_os = "macos"))]
    pub fn is_temperature(&self) -> bool {
        self.fmt.starts_with("IK")
    }

    /// If the sysctl is a structure, return the structure type string.
    ///
    /// Checks whether the format string starts with `S,` and returns the rest
    /// of the format string or None if the format String does not have a struct
    /// hint.
    pub fn struct_type(&self) -> Option<String> {
        if !self.fmt.starts_with("S,") {
            return None;
        }

        Some(self.fmt[2..].into())
    }
}

#[derive(Debug, Fail)]
pub enum SysctlError {
    #[fail(display = "no matching type for value")]
    #[cfg(not(target_os = "macos"))]
    UnknownType,

    #[fail(display = "Error extracting value")]
    ExtractionError,

    #[fail(display = "IO Error: {}", _0)]
    IoError(#[cause] io::Error),

    #[fail(display = "Error parsing UTF-8 data: {}", _0)]
    Utf8Error(#[cause] str::Utf8Error),

    #[fail(display = "Value is not readable")]
    NoReadAccess,

    #[fail(display = "Value is not writeable")]
    NoWriteAccess,

    #[fail(
        display = "sysctl returned a short read: read {} bytes, while a size of {} was reported",
        read,
        reported
    )]
    ShortRead { read: usize, reported: usize },
}

/// A custom type for temperature sysctls.
///
/// # Example
/// ```
/// extern crate sysctl;
/// #[cfg(not(target_os = "macos"))]
/// fn main() {
/// #   let ctl = match sysctl::Ctl::new("dev.cpu.0.temperature") {
/// #       Ok(c) => c,
/// #       Err(e) => {
/// #           println!("Couldn't get dev.cpu.0.temperature: {}", e);
/// #           return;
/// #       }
/// #   };
///     if let Ok(sysctl::CtlValue::Temperature(val)) = ctl.value() {
///         println!("Temperature: {:.2}K, {:.2}F, {:.2}C",
///                  val.kelvin(),
///                  val.fahrenheit(),
///                  val.celsius());
///     } else {
///         panic!("Error, not a temperature ctl!")
///     }
/// }
/// ```
/// Not available on MacOS
#[cfg(not(target_os = "macos"))]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Temperature {
    value: f32, // Kelvin
}
#[cfg(not(target_os = "macos"))]
impl Temperature {
    pub fn kelvin(&self) -> f32 {
        self.value
    }
    pub fn celsius(&self) -> f32 {
        self.value - 273.15
    }
    pub fn fahrenheit(&self) -> f32 {
        1.8 * self.celsius() + 32.0
    }
}

#[cfg(not(target_os = "macos"))]
fn name2oid(name: &str) -> Result<Vec<c_int>, SysctlError> {
    // Request command for OID
    let oid: [c_int; 2] = [0, 3];

    let mut len: usize = CTL_MAXNAME as usize * mem::size_of::<c_int>();

    // We get results in this vector
    let mut res: Vec<c_int> = vec![0; CTL_MAXNAME as usize];

    let ret = unsafe {
        sysctl(
            oid.as_ptr(),
            2,
            res.as_mut_ptr() as *mut c_void,
            &mut len,
            name.as_ptr() as *const c_void,
            name.len(),
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(io::Error::last_os_error()));
    }

    // len is in bytes, convert to number of c_ints
    len /= mem::size_of::<c_int>();

    // Trim result vector
    res.truncate(len);

    Ok(res)
}

#[cfg(target_os = "macos")]
fn name2oid(name: &str) -> Result<Vec<c_int>, SysctlError> {
    // Request command for OID
    let mut oid: [c_int; 2] = [0, 3];

    let mut len: usize = CTL_MAXNAME as usize * mem::size_of::<c_int>();

    // We get results in this vector
    let mut res: Vec<c_int> = vec![0; CTL_MAXNAME as usize];

    let ret = unsafe {
        sysctl(
            oid.as_mut_ptr(),
            2,
            res.as_mut_ptr() as *mut c_void,
            &mut len,
            name.as_ptr() as *mut c_void,
            name.len(),
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(io::Error::last_os_error()));
    }

    // len is in bytes, convert to number of c_ints
    len /= mem::size_of::<c_int>();

    // Trim result vector
    res.truncate(len);

    Ok(res)
}

#[cfg(not(target_os = "macos"))]
fn oidfmt(oid: &[c_int]) -> Result<CtlInfo, SysctlError> {
    // Request command for type info
    let mut qoid: Vec<c_int> = vec![0, 4];
    qoid.extend(oid);

    // Store results here
    let mut buf: [c_uchar; BUFSIZ as usize] = [0; BUFSIZ as usize];
    let mut buf_len = mem::size_of_val(&buf);
    let ret = unsafe {
        sysctl(
            qoid.as_ptr(),
            qoid.len() as u32,
            buf.as_mut_ptr() as *mut c_void,
            &mut buf_len,
            ptr::null(),
            0,
        )
    };
    if ret != 0 {
        return Err(SysctlError::IoError(io::Error::last_os_error()));
    }

    // 'Kind' is the first 32 bits of result buffer
    let kind = LittleEndian::read_u32(&buf);

    // 'Type' is the first 4 bits of 'Kind'
    let ctltype_val = kind & CTLTYPE as u32;

    // 'fmt' is after 'Kind' in result buffer
    let fmt: String = match str::from_utf8(&buf[mem::size_of::<u32>()..buf_len]) {
        Ok(x) => x.to_owned(),
        Err(e) => return Err(SysctlError::Utf8Error(e)),
    };

    let s = CtlInfo {
        ctl_type: CtlType::from(ctltype_val),
        fmt: fmt,
        flags: kind,
    };
    Ok(s)
}

#[cfg(not(target_os = "macos"))]
fn temperature(info: &CtlInfo, val: &Vec<u8>) -> Result<CtlValue, SysctlError> {
    let prec: u32 = {
        match info.fmt.len() {
            l if l > 2 => match info.fmt[2..3].parse::<u32>() {
                Ok(x) if x <= 9 => x,
                _ => 1,
            },
            _ => 1,
        }
    };

    let base = 10u32.pow(prec) as f32;

    let make_temp = move |f: f32| -> Result<CtlValue, SysctlError> {
        Ok(CtlValue::Temperature(Temperature { value: f / base }))
    };

    match info.ctl_type {
        CtlType::Int => make_temp(LittleEndian::read_i32(&val) as f32),
        CtlType::S64 => make_temp(LittleEndian::read_u64(&val) as f32),
        CtlType::Uint => make_temp(LittleEndian::read_u32(&val) as f32),
        CtlType::Long => make_temp(LittleEndian::read_i64(&val) as f32),
        CtlType::Ulong => make_temp(LittleEndian::read_u64(&val) as f32),
        CtlType::U64 => make_temp(LittleEndian::read_u64(&val) as f32),
        CtlType::U8 => make_temp(val[0] as u8 as f32),
        CtlType::U16 => make_temp(LittleEndian::read_u16(&val) as f32),
        CtlType::S8 => make_temp(val[0] as i8 as f32),
        CtlType::S16 => make_temp(LittleEndian::read_i16(&val) as f32),
        CtlType::S32 => make_temp(LittleEndian::read_i32(&val) as f32),
        CtlType::U32 => make_temp(LittleEndian::read_u32(&val) as f32),
        _ => Err(SysctlError::UnknownType),
    }
}

#[cfg(target_os = "macos")]
fn oidfmt(oid: &[c_int]) -> Result<CtlInfo, SysctlError> {
    // Request command for type info
    let mut qoid: Vec<c_int> = vec![0, 4];
    qoid.extend(oid);

    // Store results here
    let mut buf: [c_uchar; BUFSIZ as usize] = [0; BUFSIZ as usize];
    let mut buf_len = mem::size_of_val(&buf);
    let ret = unsafe {
        sysctl(
            qoid.as_mut_ptr(),
            qoid.len() as u32,
            buf.as_mut_ptr() as *mut c_void,
            &mut buf_len,
            ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        return Err(SysctlError::IoError(io::Error::last_os_error()));
    }

    // 'Kind' is the first 32 bits of result buffer
    let kind = LittleEndian::read_u32(&buf);

    // 'Type' is the first 4 bits of 'Kind'
    let ctltype_val = kind & CTLTYPE as u32;

    // 'fmt' is after 'Kind' in result buffer
    let fmt: String = match str::from_utf8(&buf[mem::size_of::<u32>()..buf_len]) {
        Ok(x) => x.to_owned(),
        Err(e) => return Err(SysctlError::Utf8Error(e)),
    };

    let s = CtlInfo {
        ctl_type: CtlType::from(ctltype_val),
        fmt: fmt,
        flags: kind,
    };
    Ok(s)
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
///     println!("Value: {:?}", sysctl::value("kern.osrevision"));
/// }
/// ```
#[cfg(not(target_os = "macos"))]
pub fn value(name: &str) -> Result<CtlValue, SysctlError> {
    match name2oid(name) {
        Ok(v) => value_oid(&v),
        Err(e) => Err(e),
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
///     println!("Value: {:?}", sysctl::value("kern.osrevision"));
/// }
/// ```
#[cfg(target_os = "macos")]
pub fn value(name: &str) -> Result<CtlValue, SysctlError> {
    match name2oid(name) {
        Ok(mut v) => value_oid(&mut v),
        Err(e) => Err(e),
    }
}

/// Takes an OID as argument and returns a result
/// containing the sysctl value if success, or a SysctlError
/// on failure
///
/// # Example
/// ```
/// extern crate sysctl;
/// extern crate libc;
///
/// fn main() {
///     let mut oid = vec![libc::CTL_KERN, libc::KERN_OSREV];
///     println!("Value: {:?}", sysctl::value_oid(&oid));
/// }
/// ```
#[cfg(not(target_os = "macos"))]
pub fn value_oid(oid: &Vec<i32>) -> Result<CtlValue, SysctlError> {
    let info: CtlInfo = try!(oidfmt(&oid));

    // Check if the value is readable
    if !(info.flags & CTLFLAG_RD == CTLFLAG_RD) {
        return Err(SysctlError::NoReadAccess);
    }

    // First get size of value in bytes
    let mut val_len = 0;
    let ret = unsafe {
        sysctl(
            oid.as_ptr(),
            oid.len() as u32,
            ptr::null_mut(),
            &mut val_len,
            ptr::null(),
            0,
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(io::Error::last_os_error()));
    }

    // If the length reported is shorter than the type we will convert it into,
    // LittleEndian::read_* will panic. Therefore, expand the value length to at
    // Least the size of the value.
    let val_minsize = cmp::max(val_len, info.ctl_type.min_type_size());

    // Then get value
    let mut val: Vec<c_uchar> = vec![0; val_minsize];
    let mut new_val_len = val_len;
    let ret = unsafe {
        sysctl(
            oid.as_ptr(),
            oid.len() as u32,
            val.as_mut_ptr() as *mut c_void,
            &mut new_val_len,
            ptr::null(),
            0,
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(io::Error::last_os_error()));
    }

    // Confirm that we did not read out of bounds
    assert!(new_val_len <= val_len);
    // Confirm that we got the bytes we requested
    if new_val_len < val_len {
        return Err(SysctlError::ShortRead {
            read: new_val_len,
            reported: val_len,
        });
    }

    // Special treatment for temperature ctls.
    if info.is_temperature() {
        return temperature(&info, &val);
    }

    // Wrap in Enum and return
    match info.ctl_type {
        CtlType::None => Ok(CtlValue::None),
        CtlType::Node => Ok(CtlValue::Node(val)),
        CtlType::Int => Ok(CtlValue::Int(LittleEndian::read_i32(&val))),
        CtlType::String => match val.len() {
            0 => Ok(CtlValue::String("".to_string())),
            l => str::from_utf8(&val[..l - 1])
                .map_err(|e| SysctlError::Utf8Error(e))
                .map(|s| CtlValue::String(s.into())),
        },
        CtlType::S64 => Ok(CtlValue::S64(LittleEndian::read_u64(&val))),
        CtlType::Struct => Ok(CtlValue::Struct(val)),
        CtlType::Uint => Ok(CtlValue::Uint(LittleEndian::read_u32(&val))),
        CtlType::Long => Ok(CtlValue::Long(LittleEndian::read_i64(&val))),
        CtlType::Ulong => Ok(CtlValue::Ulong(LittleEndian::read_u64(&val))),
        CtlType::U64 => Ok(CtlValue::U64(LittleEndian::read_u64(&val))),
        CtlType::U8 => Ok(CtlValue::U8(val[0])),
        CtlType::U16 => Ok(CtlValue::U16(LittleEndian::read_u16(&val))),
        CtlType::S8 => Ok(CtlValue::S8(val[0] as i8)),
        CtlType::S16 => Ok(CtlValue::S16(LittleEndian::read_i16(&val))),
        CtlType::S32 => Ok(CtlValue::S32(LittleEndian::read_i32(&val))),
        CtlType::U32 => Ok(CtlValue::U32(LittleEndian::read_u32(&val))),
        _ => Err(SysctlError::UnknownType),
    }
}

/// Takes an OID as argument and returns a result
/// containing the sysctl value if success, or a SysctlError
/// on failure
///
/// # Example
/// ```
/// extern crate sysctl;
/// extern crate libc;
///
/// fn main() {
///     let mut oid = vec![libc::CTL_KERN, libc::KERN_OSREV];
///     println!("Value: {:?}", sysctl::value_oid(&mut oid));
/// }
/// ```
#[cfg(target_os = "macos")]
pub fn value_oid(oid: &mut Vec<i32>) -> Result<CtlValue, SysctlError> {
    let info: CtlInfo = try!(oidfmt(&oid));

    // Check if the value is readable
    if !(info.flags & CTLFLAG_RD == CTLFLAG_RD) {
        return Err(SysctlError::NoReadAccess);
    }

    // First get size of value in bytes
    let mut val_len = 0;
    let ret = unsafe {
        sysctl(
            oid.as_mut_ptr(),
            oid.len() as u32,
            ptr::null_mut(),
            &mut val_len,
            ptr::null_mut(),
            0,
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(io::Error::last_os_error()));
    }

    // If the length reported is shorter than the type we will convert it into,
    // LittleEndian::read_* will panic. Therefore, expand the value length to at
    // Least the size of the value.
    let val_minsize = cmp::max(val_len, info.ctl_type.min_type_size());

    // Then get value
    let mut val: Vec<c_uchar> = vec![0; val_minsize];
    let mut new_val_len = val_len;
    let ret = unsafe {
        sysctl(
            oid.as_mut_ptr(),
            oid.len() as u32,
            val.as_mut_ptr() as *mut c_void,
            &mut new_val_len,
            ptr::null_mut(),
            0,
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(io::Error::last_os_error()));
    }

    // Confirm that we did not read out of bounds
    assert!(new_val_len <= val_len);
    // Confirm that we got the bytes we requested
    if new_val_len < val_len {
        return Err(SysctlError::ShortRead {
            read: new_val_len,
            reported: val_len,
        });
    }

    // Wrap in Enum and return
    match info.ctl_type {
        CtlType::None => Ok(CtlValue::None),
        CtlType::Node => Ok(CtlValue::Node(val)),
        CtlType::Int => Ok(CtlValue::Int(LittleEndian::read_i32(&val))),
        CtlType::String => match str::from_utf8(&val[..val.len() - 1]) {
            Ok(s) => Ok(CtlValue::String(s.into())),
            Err(e) => Err(SysctlError::Utf8Error(e)),
        },
        CtlType::S64 => Ok(CtlValue::S64(LittleEndian::read_u64(&val))),
        CtlType::Struct => Ok(CtlValue::Struct(val)),
        CtlType::Uint => Ok(CtlValue::Uint(LittleEndian::read_u32(&val))),
        CtlType::Long => Ok(CtlValue::Long(LittleEndian::read_i64(&val))),
        CtlType::Ulong => Ok(CtlValue::Ulong(LittleEndian::read_u64(&val))),
        CtlType::U64 => Ok(CtlValue::U64(LittleEndian::read_u64(&val))),
        CtlType::U8 => Ok(CtlValue::U8(val[0])),
        CtlType::U16 => Ok(CtlValue::U16(LittleEndian::read_u16(&val))),
        CtlType::S8 => Ok(CtlValue::S8(val[0] as i8)),
        CtlType::S16 => Ok(CtlValue::S16(LittleEndian::read_i16(&val))),
        CtlType::S32 => Ok(CtlValue::S32(LittleEndian::read_i32(&val))),
        CtlType::U32 => Ok(CtlValue::U32(LittleEndian::read_u32(&val))),
        #[cfg(not(target_os = "macos"))]
        _ => Err(SysctlError::UnknownType),
    }
}

/// A generic function that takes a string as argument and
/// returns a result containing the sysctl value if success,
/// or a SysctlError on failure.
///
/// Can only be called for sysctls of type Opaque or Struct.
///
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
///     println!("{:?}", sysctl::value_as::<ClockInfo>("kern.clockrate"));
/// }
/// ```
#[cfg(not(target_os = "macos"))]
pub fn value_as<T>(name: &str) -> Result<Box<T>, SysctlError> {
    match name2oid(name) {
        Ok(v) => value_oid_as::<T>(&v),
        Err(e) => Err(e),
    }
}

/// A generic function that takes a string as argument and
/// returns a result containing the sysctl value if success,
/// or a SysctlError on failure.
///
/// Can only be called for sysctls of type Opaque or Struct.
///
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
///     println!("{:?}", sysctl::value_as::<ClockInfo>("kern.clockrate"));
/// }
/// ```
#[cfg(target_os = "macos")]
pub fn value_as<T>(name: &str) -> Result<Box<T>, SysctlError> {
    match name2oid(name) {
        Ok(mut v) => value_oid_as::<T>(&mut v),
        Err(e) => Err(e),
    }
}

/// A generic function that takes an OID as argument and
/// returns a result containing the sysctl value if success,
/// or a SysctlError on failure
///
/// Can only be called for sysctls of type Opaque or Struct.
///
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
/// #[cfg(not(target_os = "macos"))]
/// fn main() {
///     let oid = vec![libc::CTL_KERN, libc::KERN_CLOCKRATE];
///     println!("{:?}", sysctl::value_oid_as::<ClockInfo>(&oid));
/// }
/// #[cfg(target_os = "macos")]
/// fn main() {
///     let mut oid = vec![libc::CTL_KERN, libc::KERN_CLOCKRATE];
///     println!("{:?}", sysctl::value_oid_as::<ClockInfo>(&mut oid));
/// }
/// ```
#[cfg(not(target_os = "macos"))]
pub fn value_oid_as<T>(oid: &Vec<i32>) -> Result<Box<T>, SysctlError> {
    let val_enum = try!(value_oid(oid));

    // Some structs are apparently reported as Node so this check is invalid..
    // let ctl_type = CtlType::from(&val_enum);
    // assert_eq!(CtlType::Struct, ctl_type, "Error type is not struct/opaque");

    // TODO: refactor this when we have better clue to what's going on
    if let CtlValue::Struct(val) = val_enum {
        // Make sure we got correct data size
        assert_eq!(
            mem::size_of::<T>(),
            val.len(),
            "Error memory size mismatch. Size of struct {}, size of data retrieved {}.",
            mem::size_of::<T>(),
            val.len()
        );

        // val is Vec<u8>
        let val_array: Box<[u8]> = val.into_boxed_slice();
        let val_raw: *mut T = Box::into_raw(val_array) as *mut T;
        let val_box: Box<T> = unsafe { Box::from_raw(val_raw) };
        Ok(val_box)
    } else if let CtlValue::Node(val) = val_enum {
        // Make sure we got correct data size
        assert_eq!(
            mem::size_of::<T>(),
            val.len(),
            "Error memory size mismatch. Size of struct {}, size of data retrieved {}.",
            mem::size_of::<T>(),
            val.len()
        );

        // val is Vec<u8>
        let val_array: Box<[u8]> = val.into_boxed_slice();
        let val_raw: *mut T = Box::into_raw(val_array) as *mut T;
        let val_box: Box<T> = unsafe { Box::from_raw(val_raw) };
        Ok(val_box)
    } else {
        Err(SysctlError::ExtractionError)
    }
}

/// A generic function that takes an OID as argument and
/// returns a result containing the sysctl value if success,
/// or a SysctlError on failure
///
/// Can only be called for sysctls of type Opaque or Struct.
///
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
/// #[cfg(not(target_os = "macos"))]
/// fn main() {
///     let oid = vec![libc::CTL_KERN, libc::KERN_CLOCKRATE];
///     println!("{:?}", sysctl::value_oid_as::<ClockInfo>(&oid));
/// }
/// #[cfg(target_os = "macos")]
/// fn main() {
///     let mut oid = vec![libc::CTL_KERN, libc::KERN_CLOCKRATE];
///     println!("{:?}", sysctl::value_oid_as::<ClockInfo>(&mut oid));
/// }
/// ```
#[cfg(target_os = "macos")]
pub fn value_oid_as<T>(oid: &mut Vec<i32>) -> Result<Box<T>, SysctlError> {
    let val_enum = try!(value_oid(oid));

    // Some structs are apparently reported as Node so this check is invalid..
    // let ctl_type = CtlType::from(&val_enum);
    // assert_eq!(CtlType::Struct, ctl_type, "Error type is not struct/opaque");

    // TODO: refactor this when we have better clue to what's going on
    if let CtlValue::Struct(val) = val_enum {
        // Make sure we got correct data size
        assert_eq!(
            mem::size_of::<T>(),
            val.len(),
            "Error memory size mismatch. Size of struct {}, size of data retrieved {}.",
            mem::size_of::<T>(),
            val.len()
        );

        // val is Vec<u8>
        let val_array: Box<[u8]> = val.into_boxed_slice();
        let val_raw: *mut T = Box::into_raw(val_array) as *mut T;
        let val_box: Box<T> = unsafe { Box::from_raw(val_raw) };
        Ok(val_box)
    } else if let CtlValue::Node(val) = val_enum {
        // Make sure we got correct data size
        assert_eq!(
            mem::size_of::<T>(),
            val.len(),
            "Error memory size mismatch. Size of struct {}, size of data retrieved {}.",
            mem::size_of::<T>(),
            val.len()
        );

        // val is Vec<u8>
        let val_array: Box<[u8]> = val.into_boxed_slice();
        let val_raw: *mut T = Box::into_raw(val_array) as *mut T;
        let val_box: Box<T> = unsafe { Box::from_raw(val_raw) };
        Ok(val_box)
    } else {
        Err(SysctlError::ExtractionError)
    }
}

/// Sets the value of a sysctl.
/// Fetches and returns the new value if successful, or a SysctlError
/// on failure
///
/// # Example
/// ```
/// extern crate sysctl;
/// # extern crate libc;
///
/// fn main() {
/// #   if unsafe { libc::getuid() } == 0 {
/// #   let old_value = sysctl::value("hw.usb.debug").unwrap();
///     let new_value = sysctl::set_value("hw.usb.debug", sysctl::CtlValue::Int(1))
///         .expect("could not set sysctl value");
///     assert_eq!(new_value, sysctl::CtlValue::Int(1));
/// #   // restore old value
/// #   sysctl::set_value("hw.usb.debug", old_value);
/// #   } // getuid() == 0
/// }
/// ```
#[cfg(not(target_os = "macos"))]
pub fn set_value(name: &str, value: CtlValue) -> Result<CtlValue, SysctlError> {
    let oid = try!(name2oid(name));
    set_oid_value(&oid, value)
}

/// Sets the value of a sysctl.
/// Fetches and returns the new value if successful, or a SysctlError
/// on failure
///
/// # Example
/// ```ignore
/// extern crate sysctl;
///
/// fn main() {
///     println!("{:?}", sysctl::set_value("hw.usb.debug", sysctl::CtlValue::Int(1)));
/// }
/// ```
#[cfg(target_os = "macos")]
pub fn set_value(name: &str, value: CtlValue) -> Result<CtlValue, SysctlError> {
    let mut oid = try!(name2oid(name));
    set_oid_value(&mut oid, value)
}

#[cfg(not(target_os = "macos"))]
pub fn set_oid_value(oid: &Vec<c_int>, value: CtlValue) -> Result<CtlValue, SysctlError> {
    let info: CtlInfo = try!(oidfmt(&oid));

    // Check if the value is writeable
    if !(info.flags & CTLFLAG_WR == CTLFLAG_WR) {
        return Err(SysctlError::NoWriteAccess);
    }

    let ctl_type = CtlType::from(&value);
    assert_eq!(
        info.ctl_type, ctl_type,
        "Error type mismatch. Type given {:?}, sysctl type: {:?}",
        ctl_type, info.ctl_type
    );

    // TODO rest of the types

    if let CtlValue::Int(v) = value {
        let mut bytes = vec![];
        bytes
            .write_i32::<LittleEndian>(v)
            .expect("Error parsing value to byte array");

        // Set value
        let ret = unsafe {
            sysctl(
                oid.as_ptr(),
                oid.len() as u32,
                ptr::null_mut(),
                ptr::null_mut(),
                bytes.as_ptr() as *const c_void,
                bytes.len(),
            )
        };
        if ret < 0 {
            return Err(SysctlError::IoError(io::Error::last_os_error()));
        }
    }

    // Get the new value and return for confirmation
    self::value_oid(oid)
}

#[cfg(target_os = "macos")]
pub fn set_oid_value(oid: &mut Vec<c_int>, value: CtlValue) -> Result<CtlValue, SysctlError> {
    let info: CtlInfo = try!(oidfmt(&oid));

    // Check if the value is writeable
    if !(info.flags & CTLFLAG_WR == CTLFLAG_WR) {
        return Err(SysctlError::NoWriteAccess);
    }

    let ctl_type = CtlType::from(&value);
    assert_eq!(
        info.ctl_type, ctl_type,
        "Error type mismatch. Type given {:?}, sysctl type: {:?}",
        ctl_type, info.ctl_type
    );

    // TODO rest of the types

    if let CtlValue::Int(v) = value {
        let mut bytes = vec![];
        bytes
            .write_i32::<LittleEndian>(v)
            .expect("Error parsing value to byte array");

        // Set value
        let ret = unsafe {
            sysctl(
                oid.as_mut_ptr(),
                oid.len() as u32,
                ptr::null_mut(),
                ptr::null_mut(),
                bytes.as_ptr() as *mut c_void,
                bytes.len(),
            )
        };
        if ret < 0 {
            return Err(SysctlError::IoError(io::Error::last_os_error()));
        }
    }

    // Get the new value and return for confirmation
    self::value_oid(oid)
}

/// Returns a result containing the sysctl description if success,
/// or a SysctlError on failure.
///
/// # Example
/// ```
/// extern crate sysctl;
///
/// fn main() {
///     println!("Description: {:?}", sysctl::description("kern.osrevision"));
/// }
/// ```
#[cfg(not(target_os = "macos"))]
pub fn description(name: &str) -> Result<String, SysctlError> {
    let oid: Vec<c_int> = try!(name2oid(name));
    oid2description(&oid)
}

#[cfg(not(target_os = "macos"))]
fn oid2description(oid: &Vec<c_int>) -> Result<String, SysctlError> {
    // Request command for description
    let mut qoid: Vec<c_int> = vec![0, 5];
    qoid.extend(oid);

    // Store results in u8 array
    let mut buf: [c_uchar; BUFSIZ as usize] = [0; BUFSIZ as usize];
    let mut buf_len = mem::size_of_val(&buf);
    let ret = unsafe {
        sysctl(
            qoid.as_ptr(),
            qoid.len() as u32,
            buf.as_mut_ptr() as *mut c_void,
            &mut buf_len,
            ptr::null(),
            0,
        )
    };
    if ret != 0 {
        return Err(SysctlError::IoError(io::Error::last_os_error()));
    }

    // Use buf_len - 1 so that we remove the trailing NULL
    match str::from_utf8(&buf[..buf_len - 1]) {
        Ok(s) => Ok(s.to_owned()),
        Err(e) => Err(SysctlError::Utf8Error(e)),
    }
}
//NOT WORKING ON MacOS
// #[cfg(target_os = "macos")]
// pub fn description(name: &str) -> Result<String, String> {

//     let oid: Vec<c_int> = try!(name2oid(name));

//     // Request command for description
//     let mut qoid: Vec<c_int> = vec![0, 5];
//     qoid.extend(oid);

//     // Store results in u8 array
//     let mut buf: [c_uchar; BUFSIZ as usize] = [0; BUFSIZ as usize];
//     let mut buf_len = mem::size_of_val(&buf);
//     let ret = unsafe {
//         sysctl(qoid.as_mut_ptr(),
//                qoid.len() as u32,
//                buf.as_mut_ptr() as *mut c_void,
//                &mut buf_len,
//                ptr::null_mut(),
//                0)
//     };
//     if ret != 0 {
//         return Err(errno_string());
//     }

//     // Use buf_len - 1 so that we remove the trailing NULL
//     match str::from_utf8(&buf[..buf_len - 1]) {
//         Ok(s) => Ok(s.to_owned()),
//         Err(e) => Err(format!("{}", e)),
//     }
// }

#[cfg(not(target_os = "macos"))]
fn oid2name(oid: &Vec<c_int>) -> Result<String, SysctlError> {
    // Request command for name
    let mut qoid: Vec<c_int> = vec![0, 1];
    qoid.extend(oid);

    // Store results in u8 array
    let mut buf: [c_uchar; BUFSIZ as usize] = [0; BUFSIZ as usize];
    let mut buf_len = mem::size_of_val(&buf);
    let ret = unsafe {
        sysctl(
            qoid.as_ptr(),
            qoid.len() as u32,
            buf.as_mut_ptr() as *mut c_void,
            &mut buf_len,
            ptr::null(),
            0,
        )
    };
    if ret != 0 {
        return Err(SysctlError::IoError(io::Error::last_os_error()));
    }

    // Use buf_len - 1 so that we remove the trailing NULL
    match str::from_utf8(&buf[..buf_len - 1]) {
        Ok(s) => Ok(s.to_owned()),
        Err(e) => Err(SysctlError::Utf8Error(e)),
    }
}

#[cfg(target_os = "macos")]
fn oid2name(oid: &Vec<c_int>) -> Result<String, SysctlError> {
    // Request command for name
    let mut qoid: Vec<c_int> = vec![0, 1];
    qoid.extend(oid);

    // Store results in u8 array
    let mut buf: [c_uchar; BUFSIZ as usize] = [0; BUFSIZ as usize];
    let mut buf_len = mem::size_of_val(&buf);
    let ret = unsafe {
        sysctl(
            qoid.as_mut_ptr(),
            qoid.len() as u32,
            buf.as_mut_ptr() as *mut c_void,
            &mut buf_len,
            ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        return Err(SysctlError::IoError(io::Error::last_os_error()));
    }

    // Use buf_len - 1 so that we remove the trailing NULL
    match str::from_utf8(&buf[..buf_len - 1]) {
        Ok(s) => Ok(s.to_owned()),
        Err(e) => Err(SysctlError::Utf8Error(e)),
    }
}

/// Get the next OID.
#[cfg(not(target_os = "macos"))]
pub fn next_oid(oid: &Vec<c_int>) -> Result<Option<Vec<c_int>>, SysctlError> {
    // Request command for next oid
    let mut qoid: Vec<c_int> = vec![0, 2];
    qoid.extend(oid);

    let mut len: usize = CTL_MAXNAME as usize * mem::size_of::<c_int>();

    // We get results in this vector
    let mut res: Vec<c_int> = vec![0; CTL_MAXNAME as usize];

    let ret = unsafe {
        sysctl(
            qoid.as_ptr(),
            qoid.len() as u32,
            res.as_mut_ptr() as *mut c_void,
            &mut len,
            ptr::null(),
            0,
        )
    };
    if ret != 0 {
        let e = io::Error::last_os_error();

        if e.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(SysctlError::IoError(e));
    }

    // len is in bytes, convert to number of c_ints
    len /= mem::size_of::<c_int>();

    // Trim result vector
    res.truncate(len);

    Ok(Some(res))
}

/// Get the next OID.
#[cfg(target_os = "macos")]
pub fn next_oid(oid: &Vec<c_int>) -> Result<Option<Vec<c_int>>, SysctlError> {
    // Request command for next oid
    let mut qoid: Vec<c_int> = vec![0, 2];
    qoid.extend(oid);

    let mut len: usize = CTL_MAXNAME as usize * mem::size_of::<c_int>();

    // We get results in this vector
    let mut res: Vec<c_int> = vec![0; CTL_MAXNAME as usize];

    let ret = unsafe {
        sysctl(
            qoid.as_mut_ptr(),
            qoid.len() as u32,
            res.as_mut_ptr() as *mut c_void,
            &mut len,
            ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        let e = io::Error::last_os_error();

        if e.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(SysctlError::IoError(e));
    }

    // len is in bytes, convert to number of c_ints
    len /= mem::size_of::<c_int>();

    // Trim result vector
    res.truncate(len);

    Ok(Some(res))
}

/// This struct represents a system control.
#[derive(Debug, Clone, PartialEq)]
pub struct Ctl {
    pub oid: Vec<c_int>,
}

impl FromStr for Ctl {
    type Err = SysctlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let oid = name2oid(s)?;

        Ok(Ctl { oid })
    }
}

impl Ctl {
    /// Construct a Ctl from the name.
    ///
    /// This is just a wrapper around `Ctl::from_str`.
    ///
    /// # Example
    ///
    /// ```
    /// extern crate sysctl;
    /// use sysctl::Ctl;
    ///
    /// let ctl = Ctl::new("kern.osrelease");
    /// ```
    pub fn new(name: &str) -> Result<Self, SysctlError> {
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
    pub fn name(self: &Self) -> Result<String, SysctlError> {
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
    pub fn value_type(self: &Self) -> Result<CtlType, SysctlError> {
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
    pub fn description(self: &Self) -> Result<String, SysctlError> {
        oid2description(&self.oid)
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
    pub fn value(self: &Self) -> Result<CtlValue, SysctlError> {
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
    pub fn value(self: &Self) -> Result<CtlValue, SysctlError> {
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
    #[cfg(not(target_os = "macos"))]
    pub fn value_as<T>(self: &Self) -> Result<Box<T>, SysctlError> {
        value_oid_as::<T>(&self.oid)
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
    #[cfg(target_os = "macos")]
    pub fn value_as<T>(self: &Self) -> Result<Box<T>, SysctlError> {
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
    pub fn set_value(self: &Self, value: CtlValue) -> Result<CtlValue, SysctlError> {
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
    pub fn set_value(self: &Self, value: CtlValue) -> Result<CtlValue, SysctlError> {
        let mut oid = self.oid.clone();
        set_oid_value(&mut oid, value)
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
    pub fn flags(self: &Self) -> Result<CtlFlags, SysctlError> {
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
    pub fn info(self: &Self) -> Result<CtlInfo, SysctlError> {
        Ok(oidfmt(&self.oid)?)
    }
}

/// An iterator over Sysctl entries.
pub struct CtlIter {
    // if we are iterating over a Node, only include OIDs
    // starting with this base. Set to None if iterating over all
    // OIDs.
    base: Ctl,
    current: Ctl,
}

impl CtlIter {
    /// Return an iterator over the complete sysctl tree.
    pub fn root() -> Self {
        CtlIter {
            base: Ctl { oid: vec![] },
            current: Ctl { oid: vec![1] },
        }
    }

    /// Return an iterator over all sysctl entries below the given node.
    pub fn below(node: Ctl) -> Self {
        CtlIter {
            base: node.clone(),
            current: node,
        }
    }
}

impl Iterator for CtlIter {
    type Item = Result<Ctl, SysctlError>;

    fn next(&mut self) -> Option<Self::Item> {
        let oid = match next_oid(&self.current.oid) {
            Ok(Some(o)) => o,
            Err(e) => return Some(Err(e)),
            Ok(None) => return None,
        };

        // We continue iterating as long as the oid starts with the base
        let cont = oid.starts_with(&self.base.oid);

        self.current = Ctl { oid };

        match cont {
            true => Some(Ok(self.current.clone())),
            false => None,
        }
    }
}

/// Ctl implements the IntoIterator trait to allow for easy iteration
/// over nodes.
///
/// # Example
///
/// ```
/// extern crate sysctl;
/// use sysctl::Ctl;
///
/// let kern = Ctl::new("kern");
/// for ctl in kern {
///     let name = ctl.name().expect("could not get name");
///     println!("{}", name);
/// }
/// ```
impl IntoIterator for Ctl {
    type Item = Result<Ctl, SysctlError>;
    type IntoIter = CtlIter;

    fn into_iter(self: Self) -> Self::IntoIter {
        CtlIter::below(self)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::process::Command;

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn ctl_mib() {
        let oid = name2oid("kern.proc.pid").unwrap();
        assert_eq!(oid.len(), 3);
        assert_eq!(oid[0], libc::CTL_KERN);
        assert_eq!(oid[1], libc::KERN_PROC);
        assert_eq!(oid[2], libc::KERN_PROC_PID);
    }

    #[test]
    fn ctl_name() {
        let oid = vec![libc::CTL_KERN, libc::KERN_OSREV];
        let name = oid2name(&oid).expect("Could not get name of kern.osrevision sysctl.");

        assert_eq!(name, "kern.osrevision");

        let ctl = Ctl { oid };
        let name = ctl
            .name()
            .expect("Could not get name of kern.osrevision sysctl.");
        assert_eq!(name, "kern.osrevision");
    }

    #[test]
    fn ctl_type() {
        let oid = name2oid("kern").unwrap();
        let fmt = oidfmt(&oid).unwrap();
        assert_eq!(fmt.ctl_type, CtlType::Node);
        let kern = Ctl::new("kern").expect("Could not get kern node");
        let value_type = kern.value_type().expect("Could not get kern value type");
        assert_eq!(value_type, CtlType::Node);

        let oid = name2oid("kern.osrelease").unwrap();
        let fmt = oidfmt(&oid).unwrap();
        assert_eq!(fmt.ctl_type, CtlType::String);
        let osrelease = Ctl::new("kern.osrelease").expect("Could not get kern.osrelease sysctl");
        let value_type = osrelease
            .value_type()
            .expect("Could notget kern.osrelease value type");
        assert_eq!(value_type, CtlType::String);

        let oid = name2oid("kern.osrevision").unwrap();
        let fmt = oidfmt(&oid).unwrap();
        assert_eq!(fmt.ctl_type, CtlType::Int);
        let osrevision = Ctl::new("kern.osrevision").expect("Could not get kern.osrevision sysctl");
        let value_type = osrevision
            .value_type()
            .expect("Could notget kern.osrevision value type");
        assert_eq!(value_type, CtlType::Int);
    }

    #[test]
    fn ctl_flags() {
        let oid = name2oid("kern.osrelease").unwrap();
        let fmt = oidfmt(&oid).unwrap();

        assert_eq!(fmt.flags & CTLFLAG_RD, CTLFLAG_RD);
        assert_eq!(fmt.flags & CTLFLAG_WR, 0);
    }

    #[test]
    fn ctl_value_int() {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("kern.osrevision")
            .output()
            .expect("failed to execute process");
        let rev_str = String::from_utf8_lossy(&output.stdout);
        let rev = rev_str.trim().parse::<i32>().unwrap();
        let n = match value("kern.osrevision") {
            Ok(CtlValue::Int(n)) => n,
            Ok(_) => 0,
            Err(_) => 0,
        };
        assert_eq!(n, rev);

        let ctl = Ctl::new("kern.osrevision").expect("Could not get kern.osrevision sysctl.");
        let n = match ctl.value() {
            Ok(CtlValue::Int(n)) => n,
            Ok(_) => 0,
            Err(_) => 0,
        };
        assert_eq!(n, rev);
    }

    #[test]
    fn ctl_value_oid_int() {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("kern.osrevision")
            .output()
            .expect("failed to execute process");
        let rev_str = String::from_utf8_lossy(&output.stdout);
        let rev = rev_str.trim().parse::<i32>().unwrap();
        let n = match value_oid(&mut vec![libc::CTL_KERN, libc::KERN_OSREV]) {
            Ok(CtlValue::Int(n)) => n,
            Ok(_) => 0,
            Err(_) => 0,
        };
        assert_eq!(n, rev);
    }

    #[test]
    fn ctl_value_string() {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("kern.version")
            .output()
            .expect("failed to execute process");
        let ver = String::from_utf8_lossy(&output.stdout);
        let s = match value("kern.version") {
            Ok(CtlValue::String(s)) => s,
            _ => "...".into(),
        };
        assert_eq!(s.trim(), ver.trim());

        let kernversion = Ctl::new("kern.version").unwrap();
        let s = match kernversion.value() {
            Ok(CtlValue::String(s)) => s,
            _ => "...".into(),
        };
        assert_eq!(s.trim(), ver.trim());
    }

    #[test]
    fn ctl_struct_type() {
        let info = CtlInfo {
            ctl_type: CtlType::Int,
            fmt: "S,TYPE".into(),
            flags: 0,
        };

        assert_eq!(info.struct_type(), Some("TYPE".into()));

        let info = CtlInfo {
            ctl_type: CtlType::Int,
            fmt: "I".into(),
            flags: 0,
        };
        assert_eq!(info.struct_type(), None);
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn ctl_description() {
        let s: String = match description("hw.ncpu") {
            Ok(s) => s,
            _ => "...".into(),
        };
        assert_ne!(s, "0");

        let ncpu = Ctl::new("hw.ncpu").expect("could not get hw.ncpu sysctl.");
        let s: String = match ncpu.description() {
            Ok(s) => s,
            _ => "...".into(),
        };
        assert_ne!(s, "0");
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn ctl_temperature_ik() {
        let info = CtlInfo {
            ctl_type: CtlType::Int,
            fmt: "IK".into(),
            flags: 0,
        };
        let mut val = vec![];
        // Default value (IK) in deciKelvin integer
        val.write_i32::<LittleEndian>(3330)
            .expect("Error parsing value to byte array");

        let t = temperature(&info, &val).unwrap();
        if let CtlValue::Temperature(tt) = t {
            assert!(tt.kelvin() - 333.0 < 0.1);
            assert!(tt.celsius() - 59.85 < 0.1);
            assert!(tt.fahrenheit() - 139.73 < 0.1);
        } else {
            assert!(false);
        }
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn ctl_temperature_ik3() {
        let info = CtlInfo {
            ctl_type: CtlType::Int,
            fmt: "IK3".into(),
            flags: 0,
        };
        let mut val = vec![];
        // Set value in milliKelvin
        val.write_i32::<LittleEndian>(333000)
            .expect("Error parsing value to byte array");

        let t = temperature(&info, &val).unwrap();
        if let CtlValue::Temperature(tt) = t {
            assert!(tt.kelvin() - 333.0 < 0.1);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn ctl_iterate_all() {
        let root = CtlIter::root();

        let all_ctls = root.into_iter().filter_map(Result::ok);

        for ctl in all_ctls {
            println!("{:?}", ctl.name());
        }
    }

    #[test]
    fn ctl_iterate() {
        let output = Command::new("sysctl")
            .arg("security")
            .output()
            .expect("failed to execute process");
        let expected = String::from_utf8_lossy(&output.stdout);

        let security = Ctl::new("security").expect("could not get security node");

        let ctls = CtlIter::below(security);
        let mut actual: Vec<String> = vec!["".to_string()];

        for ctl in ctls {
            let ctl = match ctl {
                Err(_) => {
                    continue;
                }
                Ok(s) => s,
            };

            let name = match ctl.name() {
                Ok(s) => s,
                Err(_) => {
                    continue;
                }
            };

            let value = match ctl.value() {
                Ok(s) => s,
                Err(_) => {
                    continue;
                }
            };

            let formatted = match value {
                CtlValue::None => "(none)".to_owned(),
                CtlValue::Int(i) => format!("{}", i),
                CtlValue::Uint(i) => format!("{}", i),
                CtlValue::Long(i) => format!("{}", i),
                CtlValue::Ulong(i) => format!("{}", i),
                CtlValue::U8(i) => format!("{}", i),
                CtlValue::U16(i) => format!("{}", i),
                CtlValue::U32(i) => format!("{}", i),
                CtlValue::U64(i) => format!("{}", i),
                CtlValue::S8(i) => format!("{}", i),
                CtlValue::S16(i) => format!("{}", i),
                CtlValue::S32(i) => format!("{}", i),
                CtlValue::S64(i) => format!("{}", i),
                CtlValue::Struct(_) => "(opaque struct)".to_owned(),
                CtlValue::Node(_) => "(node)".to_owned(),
                CtlValue::String(s) => s.to_owned(),
                #[cfg(not(target_os = "macos"))]
                CtlValue::Temperature(t) => format!("{} °C", t.celsius()),
            };

            match ctl.value_type().expect("could not get value type") {
                CtlType::None => {
                    continue;
                }
                CtlType::Struct => {
                    continue;
                }
                CtlType::Node => {
                    continue;
                }
                #[cfg(not(target_os = "macos"))]
                CtlType::Temperature => {
                    continue;
                }
                _ => {}
            };

            actual.push(format!("{}: {}", name, formatted));
        }
        assert_eq!(actual.join("\n").trim(), expected.trim());
    }
}
