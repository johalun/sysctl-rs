extern crate sysctl;
#[cfg(not(target_os = "macos"))]
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

    if let sysctl::CtlValue::Temperature(val) = val_enum {
        println!(
            "Temperature: {:.2}K, {:.2}F, {:.2}C",
            val.kelvin(),
            val.fahrenheit(),
            val.celsius()
        );
    } else {
        panic!("Error, not a temperature ctl!")
    }
}
#[cfg(target_os = "macos")]
fn main() {}
