// temperature.rs

use byteorder::ByteOrder;
use ctl_error::SysctlError;
use ctl_info::CtlInfo;
use ctl_type::CtlType;
use ctl_value::CtlValue;

use std::f32;

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
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Temperature {
    value: f32, // Kelvin
}
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

pub fn temperature(info: &CtlInfo, val: &Vec<u8>) -> Result<CtlValue, SysctlError> {
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
        CtlType::Int => make_temp(byteorder::LittleEndian::read_i32(&val) as f32),
        CtlType::S64 => make_temp(byteorder::LittleEndian::read_u64(&val) as f32),
        CtlType::Uint => make_temp(byteorder::LittleEndian::read_u32(&val) as f32),
        CtlType::Long => make_temp(byteorder::LittleEndian::read_i64(&val) as f32),
        CtlType::Ulong => make_temp(byteorder::LittleEndian::read_u64(&val) as f32),
        CtlType::U64 => make_temp(byteorder::LittleEndian::read_u64(&val) as f32),
        CtlType::U8 => make_temp(val[0] as u8 as f32),
        CtlType::U16 => make_temp(byteorder::LittleEndian::read_u16(&val) as f32),
        CtlType::S8 => make_temp(val[0] as i8 as f32),
        CtlType::S16 => make_temp(byteorder::LittleEndian::read_i16(&val) as f32),
        CtlType::S32 => make_temp(byteorder::LittleEndian::read_i32(&val) as f32),
        CtlType::U32 => make_temp(byteorder::LittleEndian::read_u32(&val) as f32),
        _ => Err(SysctlError::UnknownType),
    }
}
