# T480s Fingerprint Driver for Linux

Native **Rust** Linux driver (**Touch OEM Driver** `libfprint-2-tod1`) with full **Match-on-Chip (MoC)** support for the **Synaptics Metallica MIS Touch Fingerprint Reader (`06cb:009a`)** integrated into **Lenovo ThinkPad T480s**, **T480**, **X280**, and **P52s** laptops.

Unlike legacy Python wrappers (`python3-validity` / `open-fprintd`), this driver compiles into a native C-ABI shared library (`libfprint-2-tod1-synaptics009a.so`) loaded by `fprintd` through libfprint's **TOD (Touch OEM Driver)** interface.

---

## Key Features

- **Native `libfprint-2-tod1` / `fprintd` Integration**:
  - Works out of the box with `fprintd-enroll`, `fprintd-verify`, `fprintd-list`, `fprintd-delete`, **PAM** (`sudo`, GDM/LightDM login, lock screen), **GNOME Settings**, and **Fingwit**.
  - No background Python daemon and no custom `fprintd` replacement required.
- **Hardware Match-on-Chip (MoC) QMatcher & DB2 Storage**:
  - **On-chip 1:N / 1:1 verification** (`0x5e` / `0x60` / `0x62`): biometric matching is performed entirely inside the sensor's secure processor.
  - **12-stage enrollment** (`0x69` / `0x68` / `0x6b` / `0x47`) stored directly in the sensor's internal **DB2** (`StgWindsor`) flash storage bound to the Linux UID (`S-1-5-21-...-<UID>`).
  - **Single template deletion and database reset** (`0x48` `Db2DeleteObject` / `clear_storage`).
- **Hardware Encrypted TLS 1.2 Channel (`TLS_ECDH_ECDSA_WITH_AES_256_CBC_SHA256`)**:
  - Automatic extraction and decryption of host pairing credentials from the sensor's **Flash Partition 1** (`0x40`) using host DMI (`product_name` / `product_serial`) with automatic caching in `/var/lib/fprint/.synaptics_06cb_009a_pairing.json`.
  - Every USB command and response travels encrypted inside TLS 1.2 `0x17` Application Data records (`AES-256-CBC` + `HMAC-SHA256`).
- **Instant Sensor Activation (~5 ms) & Lifecycle Management**:
  - **Background pre-warming** (`dev_probe` / `dev_init`): the TLS 1.2 handshake and factory OTP calibration (`0x6f`) are initialized when `fprintd` starts. When `sudo` or the login screen prompts for a fingerprint, the sensor LED turns on in **~5 ms**.
  - An optional `--no-timeout` mode is available for fprintd service configurations that support it; keeping the TLS session warm avoids periodic sensor resets.
  - **Suspend / Resume support (`suspend` / `resume`)**: cleanly closes the active session before system sleep and automatically reconnects upon wakeup.

---

## Driver Architecture

```
fprintd (D-Bus Daemon)
  └── libfprint-2.so.2 / libfprint-2-tod.so.1
        └── libfprint-2-tod1-synaptics009a.so (This Rust driver)
              ├── src/tod.rs       → GObject FpDeviceClass subclass (type-safe TOD-1 vtable)
              ├── src/moc.rs       → QMatcher on-chip verification (0x5e..0x62) & 12-stage enrollment (0x68..0x6b)
              ├── src/capture.rs   → 2D Capacitive Capture (0x02), OTP calibration (0x6f), LED (0x39) & EP 0x83
              ├── src/db2.rs       → On-flash DB2 biometric database (0x45..0x4b), SID identity & RSA write-enable
              ├── src/protocol.rs  → Sensor session, Metallica init (0x01/0x19/0x43/0x06) & Wire Opcodes
              ├── src/tls.rs       → TLS 1.2 ECDH_ECDSA P-256 + AES-256-CBC + HMAC-SHA256 Handshake & Record Layer
              ├── src/usb.rs       → libusb transport (EP 0x01 Bulk OUT, 0x81/0x82 Bulk IN, 0x83 Interrupt IN)
              ├── src/error.rs     → Sensor status codes & scan quality mapping to FpDeviceRetry
              └── src/log.rs       → Structured logging to journalctl (SYNAPTICS_009A_LOG)
```

---

## Installation on Arch Linux / CachyOS

The `06cb:009a` sensor must have the Synaptics QMatcher firmware (`6_07f_lenovo_mis_qm.xpfwext`, v6.07.0164) in Partition 2 and factory pairing data in Partition 1. Pairing data is usually present if the sensor was initialized under Windows or with `validity-sensors-firmware`.

This driver requires **TOD (Touch OEM Driver)** support: an extension to libfprint
that loads separately installed driver plugins. Arch's standard `libfprint`
package does not provide this loader. Install the AUR package
[`libfprint-tod`](https://aur.archlinux.org/packages/libfprint-tod), which replaces
`libfprint` and provides both `libfprint-2.so.2` and `libfprint-2-tod.so.1`.

Install the build tools, USB/GLib dependencies, and `fprintd`, then build the
TOD package from the AUR. The AUR package replaces standard `libfprint`:

```bash
sudo pacman -S --needed base-devel rust pkgconf libusb glib2 fprintd
yay -S libfprint-tod
```

The `rust` package includes Cargo. If you already manage Rust through `rustup`,
omit `rust` from the pacman command and use your existing toolchain (with Rust
2024 edition support). Substitute your preferred AUR helper for `yay` if needed.

Confirm that the TOD library is present before building:

```bash
ls -l /usr/lib/libfprint-2.so.2 /usr/lib/libfprint-2-tod.so.1
```

Build and install the driver package from this repository's Arch recipe:

```bash
cd packaging/arch
makepkg -si
sudo systemctl restart fprintd
fprintd-enroll -f right-index-finger
fprintd-verify
```

The package installs the plugin into `/usr/lib/libfprint-2/tod-1`, so pacman can
track and remove it. It does not modify PAM settings or automatically restart
`fprintd`. `fprintd` is an optional package dependency used for enrollment and
authentication. The firmware requirements above also apply to this package.

If the LED flashes briefly before you touch the sensor or enrollment repeatedly
requests a retry, follow [Sensor Calibration](#sensor-calibration) below.

The recipe downloads a pinned upstream commit, applies
`arch-library-paths.patch` and `host-calibration.patch`, and uses the supplied `Cargo.lock` for reproducible
dependency versions. It builds and runs the unit tests without network access
after fetching dependencies in `prepare()`. It does not build arbitrary edits
in your working tree. If you change `build.rs`, refresh the packaging patch and
its checksum as well.

To submit to the AUR, add your maintainer details to `PKGBUILD` and copy
`PKGBUILD`, `.SRCINFO`, `arch-library-paths.patch`, `host-calibration.patch`, and `Cargo.lock` into the AUR
package repository. After changing package metadata or source checksums,
regenerate `.SRCINFO` with `makepkg --printsrcinfo > .SRCINFO` from
`packaging/arch`. These files are provided here; the package has not been
submitted to the AUR.

---

## Optional WWAN Resume Workaround (ModemManager)

Some ThinkPad WWAN modems, including the Fibocom L830-EB-00, may remain
visible on USB after suspend but fail to reappear in NetworkManager. If the
system uses **ModemManager**, restarting it after resume forces the modem's
ports to be rediscovered. This is independent of the fingerprint driver and
is not needed when using another WWAN management stack.

Create a local systemd sleep hook:

```bash
sudoedit /etc/systemd/system-sleep/90-modemmanager
```

Add:

```sh
#!/bin/sh

if [ "$1" = post ]; then
    /usr/bin/systemctl restart ModemManager.service
fi
```

Make it executable:

```bash
sudo chmod 755 /etc/systemd/system-sleep/90-modemmanager
```

This workaround only applies to systems that have and use
`ModemManager.service`; do not install it if the modem is managed directly by
another network stack.

---

## Sensor Calibration

Generate calibration for your own sensor if the LED flashes once and turns off
before you can place your finger, or enrollment keeps returning
`enroll-remove-and-retry` / `enroll-retry-scan` without making progress.

**Validated on a ThinkPad T480s with Synaptics `06cb:009a` running CachyOS:**
sensor-specific host calibration resolved this behavior. Enrollment returned
`enroll-completed`, and a subsequent `fprintd-verify` returned `verify-match`.
This confirms operation on that system; other sensors still need verification.

If the log reports a touch before you have touched the reader, followed by a
capture timeout, the sensor may be using unsuitable host calibration. A timeout
alone does not establish that the finger was removed too early. The driver logs
whether it uses a local calibration file or the bundled fallback.

Arch package release `0.1.0-2` and later includes the `synaptics009a-calibrate`
utility. It captures three iterations with the sensor **clean, dry, and entirely
uncovered**, using the host correction algorithm from
[python-validity](https://github.com/uunicorn/python-validity/blob/master/validitysensor/sensor.py).
It does not erase fingerprints, re-pair the sensor, update firmware, or write
sensor flash. It also does not regenerate the separate on-chip clean-slate
image. The host calibration file is specific to the physical sensor; do not
copy a calibration file from another laptop.

Cancel any running enrollment and install the package containing the utility.
Keep the sensor **clean, dry, and completely untouched** during the entire
calibration, including the three-second startup delay:

```bash
sudo systemctl stop fprintd
sudo synaptics009a-calibrate --output /var/lib/fprint/.synaptics_06cb_009a_calib.bin
```

For a source build, run `cargo build --release` and use
`sudo ./target/release/synaptics009a-calibrate --output /var/lib/fprint/.synaptics_06cb_009a_calib.bin`
from the repository root instead of the installed command.

Continue only after the utility reports **`Saved 13440 bytes`**:

```bash
sudo systemctl start fprintd
fprintd-enroll -f right-index-finger
fprintd-verify
```

For enrollment and verification, place your finger normally on the touch
sensor. Expected final results are `enroll-completed` and
`verify-match (done)`, respectively.

The utility refuses to overwrite an existing file. If one exists, preserve it
under another name before generating a replacement. It creates the calibration
file only after all three captures succeed; errors must be resolved before
trying enrollment again. Do not run another fingerprint client while calibrating.
The file persists across reboots and is loaded automatically by the driver;
you do not need to recalibrate on every boot or for each finger.

To undo a newly generated calibration without deleting it:

```bash
sudo systemctl stop fprintd
sudo mv -n /var/lib/fprint/.synaptics_06cb_009a_calib.bin \
    /var/lib/fprint/.synaptics_06cb_009a_calib.bin.disabled
sudo systemctl start fprintd
```

Check `journalctl -u fprintd -b` for
`Using host calibration from /var/lib/fprint/.synaptics_06cb_009a_calib.bin`
to confirm the new file is being used. Offline tests cover frame averaging,
correction arithmetic, malformed input, and capture-timeout reporting.

---

## Usage & PAM Configuration

### Enrolling, Verifying, and Deleting Fingerprints via CLI

```bash
# Enroll the right index finger (12 touches on the sensor)
fprintd-enroll -f right-index-finger

# List enrolled fingerprints
fprintd-list "$USER"

# Verify an enrolled fingerprint
fprintd-verify

# Delete all fingerprints for the current user (both from fprintd and from the sensor's DB2 flash)
fprintd-delete "$USER"
```

You can also manage fingerprints graphically via **GNOME Settings → Users → Fingerprint Login** or **Fingwit**.

### Enabling Fingerprint Authentication for `sudo` on Arch

Add this line to the `auth` section of `/etc/pam.d/sudo`, before its existing
password rule:

```pam
auth       sufficient   pam_fprintd.so
```

Keep the existing password fallback. Test in a second terminal with `sudo -k`
followed by `sudo -v`; keep an open root shell until you confirm it works. For
GDM, the lock screen, or other PAM services, see the
[ArchWiki fprint instructions](https://wiki.archlinux.org/title/Fprint).

---

## Diagnostics & Logging (`journalctl`)

The driver includes a structured logging subsystem that writes directly to the `fprintd` systemd journal:

```bash
journalctl -u fprintd -f
```

To enable verbose `DEBUG` logs (USB Bulk/Interrupt transfer timings, `0x51` sensor status events, and TLS handshake details), add the `SYNAPTICS_009A_LOG=debug` environment variable to `fprintd.service`:

```bash
sudo systemctl edit fprintd.service
```

Add:

```ini
[Service]
Environment="SYNAPTICS_009A_LOG=debug"
```

Then restart the daemon:

```bash
sudo systemctl restart fprintd
```

Supported `SYNAPTICS_009A_LOG` levels: `error`, `warn`, `info` (default), `debug`, `off`.

---

## Project Structure

| File | Description |
| :--- | :--- |
| [`Cargo.toml`](Cargo.toml) | Rust crate manifest (`cdylib` + `rlib`) |
| [`build.rs`](build.rs) | Build script for linking against `libfprint-2-tod.so.1` and `libglib-2.0.so.0` |
| [`calib_06cb_009a.bin`](calib_06cb_009a.bin) | 2D capacitive calibration blob (`13,440` bytes) for sensor `0x0199` |
| [`src/lib.rs`](src/lib.rs) | Library entrypoint and public exports |
| [`src/tod.rs`](src/tod.rs) | `libfprint-2-tod1` GObject plugin (`FpDeviceClass` vtable, pre-warming, suspend/resume) |
| [`src/moc.rs`](src/moc.rs) | On-chip QMatcher verification (`0x5e..0x62`) and 12-stage enrollment (`0x68..0x6b`) |
| [`src/capture.rs`](src/capture.rs) | 2D timeslot compiler (`0x02`), OTP calibration (`0x6f`), LED (`0x39`), and `EP 0x83` capture |
| [`src/db2.rs`](src/db2.rs) | On-flash DB2 biometric storage (`0x45..0x4b`), SID encoding, and RSA write-enable (`0x06`) |
| [`src/protocol.rs`](src/protocol.rs) | Sensor session, Metallica init sequence, `ByteReader`, and `WireOpcode` definitions |
| [`src/tls.rs`](src/tls.rs) | Custom Synaptics TLS 1.2 stack (`ECDH_ECDSA_AES256_CBC_SHA256`) and Partition 1 (`0x40`) parser |
| [`src/usb.rs`](src/usb.rs) | `rusb` (`libusb`) USB transport with cooperative cancellation on Interrupt `0x83` |
| [`src/error.rs`](src/error.rs) | Hardware status codes (`SensorStatus`) and scan error mapping |
| [`src/log.rs`](src/log.rs) | Lightweight thread-safe logger configurable via `SYNAPTICS_009A_LOG` |

---

## License

Dual-licensed under **MIT** or **LGPL-2.1-or-later**.
