# rustypot_wizard

Terminal UI to detect, inspect and configure Dynamixel and Feetech servos
over a serial bus, built on top of [`rustypot`](https://crates.io/crates/rustypot)
and [Ratatui](https://ratatui.rs/).

## Install

Pre-built binaries for Linux (x86_64 / aarch64), macOS (x86_64 / arm64) and
Windows are published on the [Releases page](https://github.com/pollen-robotics/rustypot_wizard/releases).

A one-line installer is provided per release:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/pollen-robotics/rustypot_wizard/releases/latest/download/rustypot_wizard-installer.sh | sh
```

Or via cargo:

```sh
cargo install --git https://github.com/pollen-robotics/rustypot_wizard
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
