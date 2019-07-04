// linux/ctl_iter.rs

use super::ctl::Ctl;
use super::funcs::next_ctl;
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
    direntries: Vec<walkdir::DirEntry>,
    base: String,
    cur_idx: usize,
}

impl CtlIter {
    /// Return an iterator over the complete sysctl tree.
    pub fn root() -> Self {
        let entries: Vec<walkdir::DirEntry> = walkdir::WalkDir::new("/proc/sys")
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();
        CtlIter {
            direntries: entries,
            base: "/proc/sys".to_owned(),
            cur_idx: 0,
        }
    }

    /// Return an iterator over all sysctl entries below the given node.
    pub fn below(node: Ctl) -> Self {
        let root = node.name().unwrap_or("/proc/sys".to_owned());
        let entries: Vec<walkdir::DirEntry> = walkdir::WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();
        CtlIter {
            direntries: entries,
            base: root,
            cur_idx: 0,
        }
    }
}

impl Iterator for CtlIter {
    type Item = Result<Ctl, SysctlError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.cur_idx += 1;
        if self.cur_idx >= self.direntries.len() {
            return None;
        }

        let e: &walkdir::DirEntry = &self.direntries[self.cur_idx];

        // We continue iterating as long as the oid starts with the base
        if let Some(path) = e.path().to_str() {
            if path.starts_with(&self.base) {
                Some(Ctl::new(path))
            } else {
                None
            }
        } else {
            Some(Err(SysctlError::ParseError))
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
