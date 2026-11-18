//! Generate a host calibration file without writing sensor flash.

use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use t480s_fingerprint_v2::{SensorSession, SynapticsUsbTransport};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!(
            "Usage: synaptics009a-calibrate --output NEW_FILE\n\nStop fprintd first. Keep the sensor clean, dry and completely uncovered.\nWrites a new 13440-byte host calibration file. Never overwrites an existing\nfile or changes sensor flash, firmware, pairing, or enrolled fingerprints.\nThis is an experimental diagnostic; it does not regenerate the on-chip\nclean-slate image. Hardware validation is required."
        );
        return Ok(());
    }
    if args.len() != 2 || args[0] != "--output" {
        return Err("Expected --output NEW_FILE; see --help".into());
    }
    let output = PathBuf::from(&args[1]);
    if output.try_exists()? {
        return Err(format!("Refusing to overwrite {}", output.display()).into());
    }
    eprintln!("Keep the sensor completely uncovered. Starting in 3 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(3));
    let mut session = SensorSession::new(SynapticsUsbTransport::open()?);
    let result = (|| {
        let version = session.initialize_metallica()?;
        if version.product != 0x30 {
            return Err(t480s_fingerprint_v2::DriverError::Protocol(
                "Expected Metallica product 0x30".into(),
            ));
        }
        let bundle = session.read_tls_flash_bundle()?;
        session.establish_tls_session(&bundle)?;
        session.calibrate_host()
    })();
    // Release the TLS session so fprintd can reconnect, also on capture failure.
    if let Err(error) = session.reboot_sensor() {
        eprintln!("Sensor cleanup failed: {error}");
    }
    let calibration = result?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&output)?;
    file.write_all(&calibration)?;
    file.sync_all()?;
    eprintln!(
        "Saved {} bytes to {}. No sensor flash was written.",
        calibration.len(),
        output.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Calibration failed: {error}");
        std::process::exit(1);
    }
}
