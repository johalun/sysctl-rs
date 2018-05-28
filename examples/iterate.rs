extern crate sysctl;
use std::env;

fn format_value(value: sysctl::CtlValue) -> String {
    match value {
        sysctl::CtlValue::None => "(none)".to_owned(),
        sysctl::CtlValue::Int(i) => format!("{}", i),
        sysctl::CtlValue::Uint(i) => format!("{}", i),
        sysctl::CtlValue::Long(i) => format!("{}", i),
        sysctl::CtlValue::Ulong(i) => format!("{}", i),
        sysctl::CtlValue::U8(i) => format!("{}", i),
        sysctl::CtlValue::U16(i) => format!("{}", i),
        sysctl::CtlValue::U32(i) => format!("{}", i),
        sysctl::CtlValue::U64(i) => format!("{}", i),
        sysctl::CtlValue::S8(i) => format!("{}", i),
        sysctl::CtlValue::S16(i) => format!("{}", i),
        sysctl::CtlValue::S32(i) => format!("{}", i),
        sysctl::CtlValue::S64(i) => format!("{}", i),
        sysctl::CtlValue::Struct(_) => "(opaque struct)".to_owned(),
        sysctl::CtlValue::Node(_) => "(node)".to_owned(),
        sysctl::CtlValue::String(s) => s.to_owned(),
        #[cfg(not(target_os = "macos"))]
        sysctl::CtlValue::Temperature(t) => format!("{} °C", t.celsius()),
    }
}

fn print_ctl(ctl: &sysctl::Ctl) {
    let name = ctl.name().expect("Could not get name of control");

    if let Ok(value) = ctl.value() {
        println!("{}: {}", name, format_value(value));
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let ctls = match args.len() {
        1 => sysctl::CtlIter::root().filter_map(Result::ok),
        2 => {
            let root = sysctl::Ctl::new(&args[1]).expect("Could not get given root node.");

            let value_type = root
                .value_type()
                .expect("could not get value type of given sysctl");
            if value_type != sysctl::CtlType::Node {
                print_ctl(&root);
                return;
            }

            root.into_iter().filter_map(Result::ok)
        }
        _ => panic!("more than 1 command-line argument given"),
    };

    for ctl in ctls {
        let flags = match ctl.flags() {
            Ok(f) => f,
            Err(_) => continue,
        };

        if !flags.contains(sysctl::CtlFlags::SKIP) {
            print_ctl(&ctl);
        }
    }
}
