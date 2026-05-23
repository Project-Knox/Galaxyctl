use rusb::{Context, UsbContext};
use std::io::{Read, Write};

pub fn run(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let context = Context::new()?;
    let devices = context.devices()?;
    let mut found = false;

    for device in devices.iter() {
        if let Ok(device_desc) = device.device_descriptor() {
            if device_desc.vendor_id() == 0x04e8 {
                found = true;
                let pid = device_desc.product_id();
                
                let mut num_interfaces = 0;
                let mut has_acm = false;
                if let Ok(config_desc) = device.config_descriptor(0) {
                    num_interfaces = config_desc.num_interfaces();
                    for interface in config_desc.interfaces() {
                        for interface_desc in interface.descriptors() {
                            if interface_desc.class_code() == 2 && interface_desc.sub_class_code() == 2 {
                                has_acm = true;
                            }
                        }
                    }
                }

                println!();
                println!("Device: {:04x}:{:04x} (Bus {:03} Device {:03})", 0x04e8, pid, device.bus_number(), device.address());

                let mut mode = "Unknown";
                let mut file_access = "Unknown";
                let mut at_support = if has_acm { "Yes" } else { "No" };

                if pid == 0x685d {
                    mode = "Download Mode";
                    file_access = "None";
                    at_support = "No";
                } else if pid == 0x6860 {
                    file_access = "Full Internal Storage";
                    if has_acm {
                        mode = "MTP + ACM";
                    } else {
                        mode = "MTP";
                    }
                } else if pid == 0x6863 {
                    mode = "RNDIS";
                    file_access = "None";
                } else if pid == 0x686c {
                    mode = "MIDI";
                    file_access = "None";
                } else if pid == 0x6865 {
                    mode = "PTP";
                    file_access = "DCIM/Pictures Only";
                }

                println!("Mode: {}", mode);
                println!("Interfaces: {}", num_interfaces);
                println!("File Access: {}", file_access);
                println!("AT Commands: {}", at_support);

                if let Ok(handle) = device.open() {
                    if let Ok(manufacturer) = handle.read_manufacturer_string_ascii(&device_desc) {
                        println!("Manufacturer: {}", manufacturer);
                    }
                    if let Ok(product) = handle.read_product_string_ascii(&device_desc) {
                        println!("Product: {}", product);
                    }
                    if let Ok(serial) = handle.read_serial_number_string_ascii(&device_desc) {
                        println!("Serial: {}", serial);
                    }

                    if at_support == "Yes" {
                        if let Ok(ports) = serialport::available_ports() {
                            for port in ports {
                                if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
                                    if info.vid == 0x04e8 && info.pid == pid {
                                        if verbose {
                                            println!("\nOpening serial port {}", port.port_name);
                                        }
                                        if let Ok(mut sp) = serialport::new(&port.port_name, 115200)
                                            .timeout(std::time::Duration::from_millis(2000))
                                            .open() 
                                        {
                                            if verbose {
                                                println!("Connection established");
                                                println!("Sending AT+DEVCONINFO command");
                                            }
                                            if sp.write_all(b"AT+DEVCONINFO\r\n").is_ok() {
                                                std::thread::sleep(std::time::Duration::from_millis(500));
                                                let mut buf = vec![0; 4096];
                                                if let Ok(len) = sp.read(&mut buf) {
                                                    if len > 0 {
                                                        let response = String::from_utf8_lossy(&buf[..len]);
                                                        
                                                        if let Some(start) = response.find("+DEVCONINFO:") {
                                                            println!("\nDevice Info:");
                                                            let data = &response[start + 12..];
                                                            let end = data.find("\n").unwrap_or(data.len());
                                                            let clean_data = data[..end].trim();
                                                            
                                                            for pair in clean_data.split(';') {
                                                                if let Some(open_paren) = pair.find('(') {
                                                                    if let Some(close_paren) = pair.find(')') {
                                                                        let key = pair[..open_paren].trim();
                                                                        let val = pair[open_paren + 1..close_paren].trim();
                                                                        
                                                                        let readable_key = match key {
                                                                            "MN" => "Model",
                                                                            "VER" => "Firmware Version",
                                                                            "PRD" => "Product Code",
                                                                            "CC" => "Country Code",
                                                                            "SN" => "Serial Number",
                                                                            "IMEI" => "IMEI",
                                                                            "UN" => "UFS/MMC ID",
                                                                            "LOCK" => "Lock Status",
                                                                            "CON" => "Connection Mode",
                                                                            "BASE" => "Baseband",
                                                                            "HIDVER" => "Hidden Version",
                                                                            "MNC" => "Mobile Network Code",
                                                                            "MCC" => "Mobile Country Code",
                                                                            "AID" => "AID",
                                                                            "OMCCODE" => "OMC Code",
                                                                            "PN" => "PN",
                                                                            "LIMIT" => "Limit Status",
                                                                            "SDP" => "SDP",
                                                                            "HVID" => "Hardware VID",
                                                                            _ => key,
                                                                        };
                                                                        
                                                                        println!("  {}: {}", readable_key, val);
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            println!("\nDevice Response:\n{}", response.trim());
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            if verbose {
                                                println!("Serial port error. (Might be a permission issue. Try with sudo)");
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if pid == 0x685d {
                        if let Ok(config_desc) = device.config_descriptor(0) {
                            let mut ep_in = 0;
                            let mut ep_out = 0;
                            let mut interface_num = 0;

                            for interface in config_desc.interfaces() {
                                for interface_desc in interface.descriptors() {
                                    let mut found_in = false;
                                    let mut found_out = false;
                                    for ep_desc in interface_desc.endpoint_descriptors() {
                                        if ep_desc.transfer_type() == rusb::TransferType::Bulk {
                                            if ep_desc.direction() == rusb::Direction::In {
                                                if !found_in {
                                                    ep_in = ep_desc.address();
                                                    found_in = true;
                                                }
                                            } else {
                                                if !found_out {
                                                    ep_out = ep_desc.address();
                                                    found_out = true;
                                                }
                                            }
                                        }
                                    }
                                    if found_in && found_out {
                                        interface_num = interface_desc.interface_number();
                                        break;
                                    }
                                }
                                if ep_in != 0 && ep_out != 0 {
                                    break;
                                }
                            }

                            if ep_in != 0 && ep_out != 0 {
                                let _ = handle.set_auto_detach_kernel_driver(true);
                                if handle.claim_interface(interface_num).is_ok() {
                                    let timeout = std::time::Duration::from_millis(5000);
                                    if verbose {
                                        println!("\nClaimed interface {}, sending DVIF command", interface_num);
                                    }
                                    if handle.write_bulk(ep_out, b"DVIF", timeout).is_ok() {
                                        let mut buf = [0u8; 16384];
                                        let mut total_data = Vec::new();
                                        let mut start_found = false;

                                        loop {
                                            if let Ok(transferred) = handle.read_bulk(ep_in, &mut buf, timeout) {
                                                if transferred > 0 {
                                                    total_data.extend_from_slice(&buf[..transferred]);
                                                    
                                                    if total_data.len() > 65536 {
                                                        break;
                                                    }

                                                    if let Ok(data_str) = std::str::from_utf8(&total_data) {
                                                        if !start_found && data_str.contains("@#") {
                                                            start_found = true;
                                                        }
                                                        if start_found {
                                                            let start_idx = data_str.find("@#").unwrap() + 2;
                                                            let search_start = &data_str[start_idx..];
                                                            if search_start.contains("@#") || search_start.contains("#@") {
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                break;
                                            }
                                        }

                                        if let Ok(data_str) = std::str::from_utf8(&total_data) {
                                            if let Some(start) = data_str.find("@#") {
                                                let sub1 = &data_str[start + 2..];
                                                let end_marker = if sub1.contains("@#") { "@#" } else { "#@" };
                                                if let Some(end) = sub1.find(end_marker) {
                                                    let clean_str = &sub1[..end];
                                                    println!("\nDevice Info:");
                                                    for pair in clean_str.split(';') {
                                                        if let Some((key, val)) = pair.split_once('=') {
                                                            println!("  {}: {}", key, val);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    let _ = handle.release_interface(interface_num);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !found {
        println!("No Galaxy device found.");
    }

    Ok(())
}
