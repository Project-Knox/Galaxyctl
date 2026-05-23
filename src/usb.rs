use rusb::{Context, UsbContext};

pub fn count_galaxy_devices() -> Result<usize, Box<dyn std::error::Error>> {
    let context = Context::new()?;
    let devices = context.devices()?;
    let mut galaxy_count = 0;

    for device in devices.iter() {
        if let Ok(device_desc) = device.device_descriptor() {
            if device_desc.vendor_id() == 0x04e8 {
                galaxy_count += 1;
            }
        }
    }

    Ok(galaxy_count)
}
