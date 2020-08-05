// unix/ctl_iter.rs

use super::ctl::Ctl;
use super::funcs::next_oid;
use ctl_error::SysctlError;

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
/// use sysctl::Sysctl;
///
/// let kern = sysctl::Ctl::new("kern");
/// for ctl in kern {
///     println!("{}", ctl.name().unwrap());
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
    use crate::Sysctl;

    #[test]
    fn ctl_iter_iterate_all() {
        let root = super::CtlIter::root();
        let all_ctls: Vec<super::Ctl> = root.into_iter().filter_map(Result::ok).collect();
        assert_ne!(all_ctls.len(), 0);
        for ctl in &all_ctls {
            println!("{:?}", ctl.name());
        }
    }

    #[test]
    fn ctl_iter_below_compare_outputs() {
        let output = std::process::Command::new("sysctl")
            .arg("security")
            .output()
            .expect("failed to execute process");
        let expected = String::from_utf8_lossy(&output.stdout);

        let security = super::Ctl::new("security").expect("could not get security node");

        let ctls = super::CtlIter::below(security);
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

            match ctl.value_type().expect("could not get value type") {
                crate::CtlType::None => {
                    continue;
                }
                crate::CtlType::Struct => {
                    continue;
                }
                crate::CtlType::Node => {
                    continue;
                }
                #[cfg(not(target_os = "macos"))]
                crate::CtlType::Temperature => {
                    continue;
                }
                _ => {}
            };

            actual.push(format!(
                "{}: {}",
                name,
                ctl.value_string().expect("could not get value as string")
            ));
        }
        assert_eq!(actual.join("\n").trim(), expected.trim());
    }
}
