// traits.rs

use consts::*;
use ctl_error::SysctlError;
use ctl_info::CtlInfo;
use ctl_type::CtlType;
use ctl_value::CtlValue;

pub trait Sysctl {
    fn new(name: &str) -> Result<Self, SysctlError>
    where
        Self: std::marker::Sized;
    fn name(&self) -> Result<String, SysctlError>;
    fn value_type(&self) -> Result<CtlType, SysctlError>;
    fn description(&self) -> Result<String, SysctlError>;
    fn value(&self) -> Result<CtlValue, SysctlError>;
    fn value_string(&self) -> Result<String, SysctlError>;
    fn value_as<T>(&self) -> Result<Box<T>, SysctlError>;
    fn set_value(&self, value: CtlValue) -> Result<CtlValue, SysctlError>;
    fn set_value_string(&self, value: &str) -> Result<String, SysctlError>;
    fn flags(&self) -> Result<CtlFlags, SysctlError>;
    fn info(&self) -> Result<CtlInfo, SysctlError>;
}
