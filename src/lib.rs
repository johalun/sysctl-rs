//! A simplified interface to the `sysctl` system call.
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
#[macro_use]
extern crate failure;
extern crate byteorder;
extern crate libc;

#[cfg(any(target_os = "android", target_os = "linux"))]
extern crate walkdir;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[path = "linux/mod.rs"]
mod sys;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
#[path = "unix/mod.rs"]
mod sys;

mod consts;
mod ctl_error;
mod ctl_info;
mod ctl_type;
mod ctl_value;
#[cfg(target_os = "freebsd")]
mod temperature;
mod traits;

pub use consts::*;
pub use ctl_error::*;
pub use ctl_info::*;
pub use ctl_type::*;
pub use ctl_value::*;
pub use sys::ctl::*;
pub use sys::ctl_iter::*;
pub use sys::funcs::*;
#[cfg(target_os = "freebsd")]
pub use temperature::*;
pub use traits::Sysctl;
