#![allow(dead_code)]
#![allow(unused_imports)]

extern crate sysctl;

// Import the trait
use sysctl::Sysctl;

#[cfg(target_os = "freebsd")]
fn main() {
    let ctl = match sysctl::Ctl::new("dev.cpu.0.temperature") {
        Ok(c) => c,
        Err(e) => {
            println!("Couldn't get dev.cpu.0.temperature: {}", e);
            return;
        }
    };

    let name = ctl.name().expect("could not get sysctl name");
    println!("Read sysctl {}", name);

    let d = ctl.description().expect("could not get description");
    println!("Description: {:?}", d);

    let val_enum = ctl.value().expect("could not get value");

    let temp = val_enum.as_temperature().unwrap();
    println!(
        "Temperature: {:.2}K, {:.2}F, {:.2}C",
        temp.kelvin(),
        temp.fahrenheit(),
        temp.celsius()
    );
}

#[cfg(not(target_os = "freebsd"))]
fn main() {
    println!("This operation is only supported on FreeBSD.");
}
