extern crate sysctl;
#[cfg(not(target_os = "macos"))]
fn main() {

    let ctl = "kern.osrevision";

    println!("\nRead sysctl {}", ctl);

    let d: String = sysctl::description(ctl).unwrap();
    println!("Description: {:?}", d);

    let val_enum = sysctl::value(ctl).unwrap();

    if let sysctl::CtlValue::Int(val) = val_enum {
        println!("Value: {}", val);
    }
}

#[cfg(target_os = "macos")]
fn main() {}
