//! A simplified interface to the `sysctl` system call.
//!
//! # Example: Get description and value
//! ```
//! extern crate sysctl;
//!
//! #[cfg(target_os = "freebsd")]
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
//! #[cfg(not(any(target_os = "macos", target_os = "freebsd")))]
//! fn main() { }
//! ```
//! # Example: Get value as struct
//! ```
//! #[cfg(any(target_os = "macos", target_os = "freebsd"))]
//! extern crate libc;
//! extern crate sysctl;
//!
//! #[cfg(any(target_os = "macos", target_os = "freebsd"))]
//! #[derive(Debug)]
//! #[repr(C)]
//! struct ClockInfo {
//!     hz: libc::c_int, /* clock frequency */
//!     tick: libc::c_int, /* micro-seconds per hz tick */
//!     spare: libc::c_int,
//!     stathz: libc::c_int, /* statistics clock frequency */
//!     profhz: libc::c_int, /* profiling clock frequency */
//! }
//!
//! #[cfg(any(target_os = "macos", target_os = "freebsd"))]
//! fn main() {
//!     println!("{:?}", sysctl::value_as::<ClockInfo>("kern.clockrate"));
//! }
//! #[cfg(not(any(target_os = "macos", target_os = "freebsd")))]
//! fn main() { }
//! ```

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
extern crate libc;
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
#[macro_use]
extern crate bitflags;
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
extern crate byteorder;
#[macro_use]
extern crate failure;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[path = "./linux.rs"]
mod sys;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
#[path = "./unix.rs"]
mod sys;

mod error;

pub use self::error::*;
pub use self::sys::*;

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
    #[cfg(target_os = "freebsd")]
    Temperature(Temperature),
}
