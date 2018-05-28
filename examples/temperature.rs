extern crate sysctl;
#[cfg(not(target_os = "macos"))]
fn main() {
    let ctl = "dev.cpu.0.temperature";
    println!("\nRead sysctl {}", ctl);

    let d = sysctl::description(ctl).unwrap();
    println!("Description: {:?}", d);

    let val_enum = sysctl::value(ctl).unwrap();

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
