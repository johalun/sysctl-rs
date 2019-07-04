// unix/ctl_iter.rs

use super::ctl::Ctl;
use super::funcs::{name2oid, next_oid, oid2description, oid2name, oidfmt};
use consts::*;
use ctl_error::SysctlError;
use ctl_info::CtlInfo;
use ctl_type::CtlType;
use ctl_value::CtlValue;
use std::str::FromStr;
use traits::Sysctl;

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
