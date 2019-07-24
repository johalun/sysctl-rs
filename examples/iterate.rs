#![allow(dead_code)]
#![allow(unused_imports)]

extern crate sysctl;

// Import the trait
use sysctl::Sysctl;

fn print_ctl(ctl: &sysctl::Ctl) {
    let name = ctl.name().expect("Could not get name of control");

    if let Ok(value) = ctl.value() {
        println!("{}: {}", name, value);
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let ctls = match args.len() {
        1 => sysctl::CtlIter::root().filter_map(Result::ok),
        2 => {
            let root = sysctl::Ctl::new(&args[1]).expect("Could not get given root node.");

            let value_type = root
                .value_type()
                .expect("Could not get value type of given sysctl");
            if value_type != sysctl::CtlType::Node {
                print_ctl(&root);
                return;
            }

            root.into_iter().filter_map(Result::ok)
        }
        _ => panic!("More than 1 command-line argument given"),
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
