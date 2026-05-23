mod usb;
mod usage;
mod cmd_detect;
mod cmd_device;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage::print_usage();
        return Ok(());
    }

    let mut command = "";
    let mut wait = false;
    let mut verbose = false;

    for arg in args.iter().skip(1) {
        if arg == "--wait" {
            wait = true;
        } else if arg == "--verbose" {
            verbose = true;
        } else if arg == "detect" || arg == "device" {
            command = arg;
        } else {
            println!("Unknown option or command: {}", arg);
            usage::print_usage();
            return Ok(());
        }
    }

    if command == "" {
        usage::print_usage();
        return Ok(());
    }

    if command == "detect" {
        cmd_detect::run(wait, verbose)?;
    } else if command == "device" {
        cmd_device::run(verbose)?;
    }

    Ok(())
}
