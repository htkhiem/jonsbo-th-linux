use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Error as AnyhowError, Result};
use hidapi::HidApi;

// Currently not configurable from CLI as there is no need.
static UPDATE_PERIOD_MS: u64 = 500;

fn get_thermal_zone(zone_type: &str) -> Result<PathBuf> {
    let mut basepath = PathBuf::new();
    basepath.push("/sys/class/thermal");
    for entry in fs::read_dir(&basepath)? {
        if let Ok(entry) = entry {
            let mut type_path = entry.path();
            type_path.push("type");
            match fs::read_to_string(type_path) {
                Ok(type_str) => {
                    if type_str.trim() == zone_type {
                        let mut temp_path = entry.path();
                        temp_path.push("temp");
                        return Ok(temp_path);
                        // return Ok(fs::read_to_string(temp_path)?.trim().parse::<f64>()? / 1000.0);
                    }
                }
                Err(e) => {
                    dbg!(e);
                }
            }
        }
    }
    return Err(AnyhowError::msg("Not found"));
}

fn main() -> Result<()> {
    // Parse cmd args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: jonsbo_th <vid:pid> [thermal_zone_type]");
        std::process::exit(1);
    }

    let target_id = &args[1];
    let zone_type = args
        .get(2)
        .map(|s| s.as_str().trim())
        .unwrap_or("x86_pkg_temp");

    // Get the corresponding thermal zone. Might change between boots so we must search by type.
    let temp_path =
        get_thermal_zone(zone_type).context("Could not find a thermal zone with the given type")?;

    // Parse "vid:pid" hex string (e.g., "5131:2007", which is the VID of the temperature display on the TH-360 AIO)
    let ids: Vec<u16> = target_id
        .split(':')
        .map(|s| u16::from_str_radix(s, 16))
        .collect::<Result<Vec<_>, _>>()
        .context("Invalid VID:PID format. Expected hex like 5131:2007")?;

    let (vid, pid) = (ids[0], ids[1]);

    let api = HidApi::new().context("Failed to initialize HID API")?;
    let device = api
        .open(vid, pid)
        .context("Could not open device. Check permissions/udev.")?;

    loop {
        // Allow the occasional failure
        if let Ok(raw_temp) = fs::read_to_string(&temp_path) {
            if let Ok(temp) = raw_temp.trim().parse::<f64>() {
                let mut buf = [0u8; 64];
                buf[0] = 0x01; // Report ID
                buf[1] = 0x02; // Command
                               // No idea but the third byte is not needed?
                buf[3] = (temp / 1000.0).max(0.0).min(99.0) as u8; // Value to display. TH-360 only has two 7-seg digits.

                device
                    .write(&buf)
                    .context("Failed to write to temperature display")?;

                std::thread::sleep(std::time::Duration::from_millis(UPDATE_PERIOD_MS));
            }
        }
    }
    Ok(())
}
