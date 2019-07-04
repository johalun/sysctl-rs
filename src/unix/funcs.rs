// unix/funcs.rs

use byteorder::{ByteOrder, WriteBytesExt};
use consts::*;
use ctl_error::*;
use ctl_info::*;
use ctl_type::*;
use ctl_value::*;

#[cfg(target_os = "freebsd")]
use temperature::*;

#[cfg(not(target_os = "macos"))]
pub fn name2oid(name: &str) -> Result<Vec<libc::c_int>, SysctlError> {
    // Request command for OID
    let oid: [libc::c_int; 2] = [0, 3];

    let mut len: usize = CTL_MAXNAME as usize * std::mem::size_of::<libc::c_int>();

    // We get results in this vector
    let mut res: Vec<libc::c_int> = vec![0; CTL_MAXNAME as usize];

    let ret = unsafe {
        libc::sysctl(
            oid.as_ptr(),
            2,
            res.as_mut_ptr() as *mut libc::c_void,
            &mut len,
            name.as_ptr() as *const libc::c_void,
            name.len(),
        )
    };
    if ret < 0 {
        let e = std::io::Error::last_os_error();
        return Err(match e.kind() {
            std::io::ErrorKind::NotFound => SysctlError::NotFound(name.into()),
            _ => SysctlError::IoError(e),
        });
    }

    // len is in bytes, convert to number of libc::c_ints
    len /= std::mem::size_of::<libc::c_int>();

    // Trim result vector
    res.truncate(len);

    Ok(res)
}

#[cfg(target_os = "macos")]
pub fn name2oid(name: &str) -> Result<Vec<libc::c_int>, SysctlError> {
    // Request command for OID
    let mut oid: [libc::c_int; 2] = [0, 3];

    let mut len: usize = CTL_MAXNAME as usize * std::mem::size_of::<libc::c_int>();

    // We get results in this vector
    let mut res: Vec<libc::c_int> = vec![0; CTL_MAXNAME as usize];

    let ret = unsafe {
        libc::sysctl(
            oid.as_mut_ptr(),
            2,
            res.as_mut_ptr() as *mut libc::c_void,
            &mut len,
            name.as_ptr() as *mut libc::c_void,
            name.len(),
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(std::io::Error::last_os_error()));
    }

    // len is in bytes, convert to number of libc::c_ints
    len /= std::mem::size_of::<libc::c_int>();

    // Trim result vector
    res.truncate(len);

    Ok(res)
}

#[cfg(not(target_os = "macos"))]
pub fn oidfmt(oid: &[libc::c_int]) -> Result<CtlInfo, SysctlError> {
    // Request command for type info
    let mut qoid: Vec<libc::c_int> = vec![0, 4];
    qoid.extend(oid);

    // Store results here
    let mut buf: [libc::c_uchar; libc::BUFSIZ as usize] = [0; libc::BUFSIZ as usize];
    let mut buf_len = std::mem::size_of_val(&buf);
    let ret = unsafe {
        libc::sysctl(
            qoid.as_ptr(),
            qoid.len() as u32,
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut buf_len,
            std::ptr::null(),
            0,
        )
    };
    if ret != 0 {
        return Err(SysctlError::IoError(std::io::Error::last_os_error()));
    }

    // 'Kind' is the first 32 bits of result buffer
    let kind = byteorder::LittleEndian::read_u32(&buf);

    // 'Type' is the first 4 bits of 'Kind'
    let ctltype_val = kind & CTLTYPE as u32;

    // 'fmt' is after 'Kind' in result buffer
    let fmt: String =
        match std::ffi::CStr::from_bytes_with_nul(&buf[std::mem::size_of::<u32>()..buf_len]) {
            Ok(x) => x.to_string_lossy().into(),
            Err(e) => return Err(SysctlError::InvalidCStr(e)),
        };

    let s = CtlInfo {
        ctl_type: CtlType::from(ctltype_val),
        fmt: fmt,
        flags: kind,
    };
    Ok(s)
}

#[cfg(target_os = "macos")]
pub fn oidfmt(oid: &[libc::c_int]) -> Result<CtlInfo, SysctlError> {
    // Request command for type info
    let mut qoid: Vec<libc::c_int> = vec![0, 4];
    qoid.extend(oid);

    // Store results here
    let mut buf: [libc::c_uchar; libc::BUFSIZ as usize] = [0; libc::BUFSIZ as usize];
    let mut buf_len = std::mem::size_of_val(&buf);
    let ret = unsafe {
        libc::sysctl(
            qoid.as_mut_ptr(),
            qoid.len() as u32,
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut buf_len,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        return Err(SysctlError::IoError(std::io::Error::last_os_error()));
    }

    // 'Kind' is the first 32 bits of result buffer
    let kind = byteorder::LittleEndian::read_u32(&buf);

    // 'Type' is the first 4 bits of 'Kind'
    let ctltype_val = kind & CTLTYPE as u32;

    // 'fmt' is after 'Kind' in result buffer
    let fmt: String = match std::str::from_utf8(&buf[std::mem::size_of::<u32>()..buf_len]) {
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
        libc::sysctl(
            oid.as_ptr(),
            oid.len() as u32,
            std::ptr::null_mut(),
            &mut val_len,
            std::ptr::null(),
            0,
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(std::io::Error::last_os_error()));
    }

    // If the length reported is shorter than the type we will convert it into,
    // byteorder::LittleEndian::read_* will panic. Therefore, expand the value length to at
    // Least the size of the value.
    let val_minsize = std::cmp::max(val_len, info.ctl_type.min_type_size());

    // Then get value
    let mut val: Vec<libc::c_uchar> = vec![0; val_minsize];
    let mut new_val_len = val_len;
    let ret = unsafe {
        libc::sysctl(
            oid.as_ptr(),
            oid.len() as u32,
            val.as_mut_ptr() as *mut libc::c_void,
            &mut new_val_len,
            std::ptr::null(),
            0,
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(std::io::Error::last_os_error()));
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
        CtlType::Int => Ok(CtlValue::Int(byteorder::LittleEndian::read_i32(&val))),
        CtlType::String => match val.len() {
            0 => Ok(CtlValue::String("".to_string())),
            l => std::str::from_utf8(&val[..l - 1])
                .map_err(|e| SysctlError::Utf8Error(e))
                .map(|s| CtlValue::String(s.into())),
        },
        CtlType::S64 => Ok(CtlValue::S64(byteorder::LittleEndian::read_u64(&val))),
        CtlType::Struct => Ok(CtlValue::Struct(val)),
        CtlType::Uint => Ok(CtlValue::Uint(byteorder::LittleEndian::read_u32(&val))),
        CtlType::Long => Ok(CtlValue::Long(byteorder::LittleEndian::read_i64(&val))),
        CtlType::Ulong => Ok(CtlValue::Ulong(byteorder::LittleEndian::read_u64(&val))),
        CtlType::U64 => Ok(CtlValue::U64(byteorder::LittleEndian::read_u64(&val))),
        CtlType::U8 => Ok(CtlValue::U8(val[0])),
        CtlType::U16 => Ok(CtlValue::U16(byteorder::LittleEndian::read_u16(&val))),
        CtlType::S8 => Ok(CtlValue::S8(val[0] as i8)),
        CtlType::S16 => Ok(CtlValue::S16(byteorder::LittleEndian::read_i16(&val))),
        CtlType::S32 => Ok(CtlValue::S32(byteorder::LittleEndian::read_i32(&val))),
        CtlType::U32 => Ok(CtlValue::U32(byteorder::LittleEndian::read_u32(&val))),
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
        libc::sysctl(
            oid.as_mut_ptr(),
            oid.len() as u32,
            std::ptr::null_mut(),
            &mut val_len,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(std::io::Error::last_os_error()));
    }

    // If the length reported is shorter than the type we will convert it into,
    // byteorder::LittleEndian::read_* will panic. Therefore, expand the value length to at
    // Least the size of the value.
    let val_minsize = cmp::max(val_len, info.ctl_type.min_type_size());

    // Then get value
    let mut val: Vec<libc::c_uchar> = vec![0; val_minsize];
    let mut new_val_len = val_len;
    let ret = unsafe {
        libc::sysctl(
            oid.as_mut_ptr(),
            oid.len() as u32,
            val.as_mut_ptr() as *mut libc::c_void,
            &mut new_val_len,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(std::io::Error::last_os_error()));
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
        CtlType::Int => Ok(CtlValue::Int(byteorder::LittleEndian::read_i32(&val))),
        CtlType::String => match std::str::from_utf8(&val[..val.len() - 1]) {
            Ok(s) => Ok(CtlValue::String(s.into())),
            Err(e) => Err(SysctlError::Utf8Error(e)),
        },
        CtlType::S64 => Ok(CtlValue::S64(byteorder::LittleEndian::read_u64(&val))),
        CtlType::Struct => Ok(CtlValue::Struct(val)),
        CtlType::Uint => Ok(CtlValue::Uint(byteorder::LittleEndian::read_u32(&val))),
        CtlType::Long => Ok(CtlValue::Long(byteorder::LittleEndian::read_i64(&val))),
        CtlType::Ulong => Ok(CtlValue::Ulong(byteorder::LittleEndian::read_u64(&val))),
        CtlType::U64 => Ok(CtlValue::U64(byteorder::LittleEndian::read_u64(&val))),
        CtlType::U8 => Ok(CtlValue::U8(val[0])),
        CtlType::U16 => Ok(CtlValue::U16(byteorder::LittleEndian::read_u16(&val))),
        CtlType::S8 => Ok(CtlValue::S8(val[0] as i8)),
        CtlType::S16 => Ok(CtlValue::S16(byteorder::LittleEndian::read_i16(&val))),
        CtlType::S32 => Ok(CtlValue::S32(byteorder::LittleEndian::read_i32(&val))),
        CtlType::U32 => Ok(CtlValue::U32(byteorder::LittleEndian::read_u32(&val))),
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
///     hz: libc::c_int, /* clock frequency */
///     tick: libc::c_int, /* micro-seconds per hz tick */
///     spare: libc::c_int,
///     stathz: libc::c_int, /* statistics clock frequency */
///     profhz: libc::c_int, /* profiling clock frequency */
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
///     hz: libc::c_int, /* clock frequency */
///     tick: libc::c_int, /* micro-seconds per hz tick */
///     spare: libc::c_int,
///     stathz: libc::c_int, /* statistics clock frequency */
///     profhz: libc::c_int, /* profiling clock frequency */
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
///     hz: libc::c_int, /* clock frequency */
///     tick: libc::c_int, /* micro-seconds per hz tick */
///     spare: libc::c_int,
///     stathz: libc::c_int, /* statistics clock frequency */
///     profhz: libc::c_int, /* profiling clock frequency */
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
            std::mem::size_of::<T>(),
            val.len(),
            "Error memory size mismatch. Size of struct {}, size of data retrieved {}.",
            std::mem::size_of::<T>(),
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
            std::mem::size_of::<T>(),
            val.len(),
            "Error memory size mismatch. Size of struct {}, size of data retrieved {}.",
            std::mem::size_of::<T>(),
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
///     hz: libc::c_int, /* clock frequency */
///     tick: libc::c_int, /* micro-seconds per hz tick */
///     spare: libc::c_int,
///     stathz: libc::c_int, /* statistics clock frequency */
///     profhz: libc::c_int, /* profiling clock frequency */
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
            std::mem::size_of::<T>(),
            val.len(),
            "Error memory size mismatch. Size of struct {}, size of data retrieved {}.",
            std::mem::size_of::<T>(),
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
            std::mem::size_of::<T>(),
            val.len(),
            "Error memory size mismatch. Size of struct {}, size of data retrieved {}.",
            std::mem::size_of::<T>(),
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
pub fn set_oid_value(oid: &Vec<libc::c_int>, value: CtlValue) -> Result<CtlValue, SysctlError> {
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
    let bytes: Vec<u8> = match value {
        CtlValue::Int(v) => {
            let mut bytes = vec![];
            bytes.write_i32::<byteorder::LittleEndian>(v)?;
            Ok(bytes)
        }
        CtlValue::String(v) => Ok(v.as_bytes().to_owned()),
        _ => Err(SysctlError::MissingImplementation),
    }?;

    let ret = unsafe {
        libc::sysctl(
            oid.as_ptr(),
            oid.len() as u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            bytes.as_ptr() as *const libc::c_void,
            bytes.len(),
        )
    };
    if ret < 0 {
        return Err(SysctlError::IoError(std::io::Error::last_os_error()));
    }

    // Get the new value and return for confirmation
    self::value_oid(oid)
}

#[cfg(target_os = "macos")]
pub fn set_oid_value(oid: &mut Vec<libc::c_int>, value: CtlValue) -> Result<CtlValue, SysctlError> {
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
            .write_i32::<byteorder::LittleEndian>(v)
            .expect("Error parsing value to byte array");

        // Set value
        let ret = unsafe {
            libc::sysctl(
                oid.as_mut_ptr(),
                oid.len() as u32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                bytes.as_ptr() as *mut libc::c_void,
                bytes.len(),
            )
        };
        if ret < 0 {
            return Err(SysctlError::IoError(std::io::Error::last_os_error()));
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
    let oid: Vec<libc::c_int> = try!(name2oid(name));
    oid2description(&oid)
}

#[cfg(target_os = "macos")]
pub fn description(name: &str) -> Result<String, SysctlError> {
    Ok("<Description not available on macOS>".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn oid2description(oid: &Vec<libc::c_int>) -> Result<String, SysctlError> {
    // Request command for description
    let mut qoid: Vec<libc::c_int> = vec![0, 5];
    qoid.extend(oid);

    // Store results in u8 array
    let mut buf: [libc::c_uchar; libc::BUFSIZ as usize] = [0; libc::BUFSIZ as usize];
    let mut buf_len = std::mem::size_of_val(&buf);
    let ret = unsafe {
        libc::sysctl(
            qoid.as_ptr(),
            qoid.len() as u32,
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut buf_len,
            std::ptr::null(),
            0,
        )
    };
    if ret != 0 {
        return Err(SysctlError::IoError(std::io::Error::last_os_error()));
    }

    // Use buf_len - 1 so that we remove the trailing NULL
    match std::str::from_utf8(&buf[..buf_len - 1]) {
        Ok(s) => Ok(s.to_owned()),
        Err(e) => Err(SysctlError::Utf8Error(e)),
    }
}
//NOT WORKING ON MacOS
// #[cfg(target_os = "macos")]
// pub fn description(name: &str) -> Result<String, String> {

//     let oid: Vec<libc::c_int> = try!(name2oid(name));

//     // Request command for description
//     let mut qoid: Vec<libc::c_int> = vec![0, 5];
//     qoid.extend(oid);

//     // Store results in u8 array
//     let mut buf: [libc::c_uchar; libc::BUFSIZ as usize] = [0; libc::BUFSIZ as usize];
//     let mut buf_len = std::mem::size_of_val(&buf);
//     let ret = unsafe {
//         libc::sysctl(qoid.as_mut_ptr(),
//                qoid.len() as u32,
//                buf.as_mut_ptr() as *mut libc::c_void,
//                &mut buf_len,
//                std::ptr::null_mut(),
//                0)
//     };
//     if ret != 0 {
//         return Err(errno_string());
//     }

//     // Use buf_len - 1 so that we remove the trailing NULL
//     match std::str::from_utf8(&buf[..buf_len - 1]) {
//         Ok(s) => Ok(s.to_owned()),
//         Err(e) => Err(format!("{}", e)),
//     }
// }

#[cfg(not(target_os = "macos"))]
pub fn oid2name(oid: &Vec<libc::c_int>) -> Result<String, SysctlError> {
    // Request command for name
    let mut qoid: Vec<libc::c_int> = vec![0, 1];
    qoid.extend(oid);

    // Store results in u8 array
    let mut buf: [libc::c_uchar; libc::BUFSIZ as usize] = [0; libc::BUFSIZ as usize];
    let mut buf_len = std::mem::size_of_val(&buf);
    let ret = unsafe {
        libc::sysctl(
            qoid.as_ptr(),
            qoid.len() as u32,
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut buf_len,
            std::ptr::null(),
            0,
        )
    };
    if ret != 0 {
        return Err(SysctlError::IoError(std::io::Error::last_os_error()));
    }

    // Use buf_len - 1 so that we remove the trailing NULL
    match std::str::from_utf8(&buf[..buf_len - 1]) {
        Ok(s) => Ok(s.to_owned()),
        Err(e) => Err(SysctlError::Utf8Error(e)),
    }
}

#[cfg(target_os = "macos")]
pub fn oid2name(oid: &Vec<libc::c_int>) -> Result<String, SysctlError> {
    // Request command for name
    let mut qoid: Vec<libc::c_int> = vec![0, 1];
    qoid.extend(oid);

    // Store results in u8 array
    let mut buf: [libc::c_uchar; libc::BUFSIZ as usize] = [0; libc::BUFSIZ as usize];
    let mut buf_len = std::mem::size_of_val(&buf);
    let ret = unsafe {
        libc::sysctl(
            qoid.as_mut_ptr(),
            qoid.len() as u32,
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut buf_len,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        return Err(SysctlError::IoError(std::io::Error::last_os_error()));
    }

    // Use buf_len - 1 so that we remove the trailing NULL
    match std::str::from_utf8(&buf[..buf_len - 1]) {
        Ok(s) => Ok(s.to_owned()),
        Err(e) => Err(SysctlError::Utf8Error(e)),
    }
}

/// Get the next OID.
#[cfg(not(target_os = "macos"))]
pub fn next_oid(oid: &Vec<libc::c_int>) -> Result<Option<Vec<libc::c_int>>, SysctlError> {
    // Request command for next oid
    let mut qoid: Vec<libc::c_int> = vec![0, 2];
    qoid.extend(oid);

    let mut len: usize = CTL_MAXNAME as usize * std::mem::size_of::<libc::c_int>();

    // We get results in this vector
    let mut res: Vec<libc::c_int> = vec![0; CTL_MAXNAME as usize];

    let ret = unsafe {
        libc::sysctl(
            qoid.as_ptr(),
            qoid.len() as u32,
            res.as_mut_ptr() as *mut libc::c_void,
            &mut len,
            std::ptr::null(),
            0,
        )
    };
    if ret != 0 {
        let e = std::io::Error::last_os_error();

        if e.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(SysctlError::IoError(e));
    }

    // len is in bytes, convert to number of libc::c_ints
    len /= std::mem::size_of::<libc::c_int>();

    // Trim result vector
    res.truncate(len);

    Ok(Some(res))
}

/// Get the next OID.
#[cfg(target_os = "macos")]
pub fn next_oid(oid: &Vec<libc::c_int>) -> Result<Option<Vec<libc::c_int>>, SysctlError> {
    // Request command for next oid
    let mut qoid: Vec<libc::c_int> = vec![0, 2];
    qoid.extend(oid);

    let mut len: usize = CTL_MAXNAME as usize * std::mem::size_of::<libc::c_int>();

    // We get results in this vector
    let mut res: Vec<libc::c_int> = vec![0; CTL_MAXNAME as usize];

    let ret = unsafe {
        libc::sysctl(
            qoid.as_mut_ptr(),
            qoid.len() as u32,
            res.as_mut_ptr() as *mut libc::c_void,
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        let e = std::io::Error::last_os_error();

        if e.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(SysctlError::IoError(e));
    }

    // len is in bytes, convert to number of libc::c_ints
    len /= std::mem::size_of::<libc::c_int>();

    // Trim result vector
    res.truncate(len);

    Ok(Some(res))
}
