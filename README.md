# rustypot_wizard

Terminal UI to detect, inspect and configure Dynamixel and Feetech servos
over a serial bus, built on top of [`rustypot`](https://crates.io/crates/rustypot)
and [Ratatui](https://ratatui.rs/).

## Install

### Linux / macOS

```sh
curl -LsSf https://github.com/pollen-robotics/rustypot_wizard/releases/latest/download/rustypot_wizard-installer.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://github.com/pollen-robotics/rustypot_wizard/releases/latest/download/rustypot_wizard-installer.ps1 | iex
```

The installer drops the binary in `~/.cargo/bin` (or `%USERPROFILE%\.cargo\bin`)
and adds it to your `PATH` if needed. No Rust toolchain required.

### Pinning a version

Replace `latest` with a tag, e.g. `download/v0.1.0/`:

```sh
curl -LsSf https://github.com/pollen-robotics/rustypot_wizard/releases/download/v0.1.0/rustypot_wizard-installer.sh | sh
```

### From source

```sh
cargo install --git https://github.com/pollen-robotics/rustypot_wizard
```

Or download a tarball/zip directly from the
[Releases page](https://github.com/pollen-robotics/rustypot_wizard/releases)
(builds: linux x86_64/aarch64, macOS x86_64/arm64, windows x86_64).

### Linux serial-port permissions

To talk to a USB-serial adapter without `sudo`, add yourself to the `dialout`
group (Debian/Ubuntu) or `uucp` (Arch), then log out/in:

```sh
sudo usermod -aG dialout "$USER"
```

## Usage

```sh
rustypot_wizard
```

Pick the brand (Dynamixel / Feetech), serial port, baud rate and protocol,
then `Enter` to scan. Use the keyboard or the mouse: click motors and
registers, drag the goal-position slider, click the torque pill to toggle.
Esc returns to the connection screen, `q` quits.

## License

Apache-2.0
