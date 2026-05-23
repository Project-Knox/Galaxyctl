use crate::usb;
use std::thread::sleep;
use std::time::Duration;

pub fn run(wait: bool, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_status = -1;

    loop {
        let count = usb::count_galaxy_devices()?;

        if count == 1 {
            println!("Galaxy device found");
            break;
        } else if count == 0 {
            if last_status != 0 {
                if !wait {
                    println!("No Galaxy device found");
                } else {
                    if verbose {
                        println!("Waiting for Galaxy device...");
                    }
                }
                last_status = 0;
            }
            if wait {
                sleep(Duration::from_millis(500));
                continue;
            } else {
                break;
            }
        } else {
            if last_status != 2 {
                println!("Multiple Galaxy devices found, please leave only one connected");
                last_status = 2;
            }
            if wait {
                sleep(Duration::from_millis(500));
                continue;
            } else {
                break;
            }
        }
    }

    Ok(())
}
