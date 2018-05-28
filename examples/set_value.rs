extern crate libc;
extern crate sysctl;

fn main() {
    assert_eq!(
        unsafe { libc::geteuid() },
        0,
        "This example must be run as root"
    );

    let ctl = sysctl::Ctl::new("hw.usb.debug").expect("could not get sysctl: hw.usb.debug");

    let name = ctl.name().expect("could not get sysctl name");
    println!("\nFlipping value of sysctl {}", name);

    let old_val_enum = ctl.value().expect("could not set sysctl value");

    if let sysctl::CtlValue::Int(old_val) = old_val_enum {
        println!("Old value: {}", old_val);

        let target_val = match old_val {
            0 => 1,
            _ => 0,
        };

        let target_val_enum = sysctl::CtlValue::Int(target_val);

        let new_val_enum = ctl.set_value(target_val_enum).expect("could not set value");

        if let sysctl::CtlValue::Int(new_val) = new_val_enum {
            if new_val == target_val {
                println!("New value succcesfully set to: {}", new_val);
            } else {
                println!("Error: Could not set new value");
            }
            println!("Restore old value");

            ctl.set_value(old_val_enum)
                .expect("could not restore old value");
        }
    }
}
