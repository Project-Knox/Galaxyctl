pub fn print_usage() {
    println!("Galaxy Control CLI");
    println!("Usage:");
    println!("  galaxyctl [OPTIONS] <COMMAND>");
    println!("");
    println!("Options:");
    println!("  --wait      Wait for a device to be connected (only for 'detect')");
    println!("  --verbose   Show detailed background operations");
    println!("  --help      Print this help message");
    println!("");
    println!("Commands:");
    println!("  detect      Check if a Galaxy device is connected");
    println!("  device      Print detailed information about the connected Galaxy device");
}
