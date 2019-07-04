#![cfg(test)]

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
