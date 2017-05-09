extern crate sysctl;
extern crate libc;

fn main() {

    assert_eq!(unsafe { libc::geteuid() },
               0,
               "This example must be run as root");

    let ctl = "hw.usb.debug";

    println!("\nFlipping value of sysctl {}", ctl);

    let old_val_enum = sysctl::value(ctl).unwrap();

    if let sysctl::CtlValue::Int(old_val) = old_val_enum {
        println!("Old value: {}", old_val);

        let l = {
            if old_val == 0 { 1 } else { 0 }
        };
        let new_val_enum = sysctl::set_value(ctl, sysctl::CtlValue::Int(l)).unwrap();

        if let sysctl::CtlValue::Int(new_val) = new_val_enum {

            if new_val == l {
                println!("New value succcesfully set to: {}", new_val);
            } else {
                println!("Error: Could not set new value");
            }
            println!("Restore old value");

            sysctl::set_value(ctl, old_val_enum).unwrap();

        }
    }
}
