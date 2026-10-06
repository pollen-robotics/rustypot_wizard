# Rustypot Wizard

<img width="2560" height="1528" alt="Capture d’écran du 2026-04-30 10-43-01" src="https://github.com/user-attachments/assets/ba2fc73c-8629-4768-a04f-6eb6b2be2b57" />

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

## Benchmark mode

From the main screen press `p` to open the **communication benchmark**. It
reuses the open bus and discovered motors to measure round-trip timing of the
operations a real control loop relies on:

| Benchmark | What it measures |
|-----------|------------------|
| Ping one motor | Latency of a single ping to the selected motor |
| Ping all motors (sweep) | Latency to ping every motor on the bus once |
| Read one position | Latency of one Present Position read |
| Read all positions (sequential) | Reading every motor with individual reads |
| Read all positions (sync) | One `sync_read` for all motors — compare vs. sequential |
| R/W loop, hold (sequential) | Per motor: write the resting position (no motion), then read — try this first |
| R/W loop, hold (sync) | One `sync_write` of resting positions (no motion) + one `sync_read` |
| R/W loop, sine (sequential) | Per motor: write a small sine goal, then read position |
| R/W loop, sine (sync) | One `sync_write` of sine goals + one `sync_read` — the fast control loop |

Each benchmark runs for a fixed **duration** (default 10 s, adjustable in the
UI) and reports the number of cycles completed, min / mean / p50 / p95 / max /
std latency, the achievable rate in Hz, and a count of failed transactions
(read/write/ping errors), so a flaky cable or adapter shows up immediately. For
multi-motor benchmarks a "per motor" figure is also shown.

Controls: `↑↓` select a benchmark, `Enter` run it, `a` run all back-to-back,
`[` / `]` change the run duration, `-` / `+` change the sine amplitude, `Esc`
returns to the configurator.

All four R/W loop benchmarks capture each motor's resting position first and
restore it when finished. The **hold** variants write that resting position
back unchanged — they exercise the complete write+read loop **without commanding
any motion**, so they are the safe ones to try first. The **sine** variants add a
small offset that follows `home + A·sin(2π·f·t + φ)`: amplitude `A` defaults to
±5° (adjustable in the UI with `-`/`+` and shown in the detail pane), frequency
`f` is 0.5 Hz, and each motor gets a phase `φ = 2π·i/n` so the bus moves as a
travelling wave. The sine runs **enable torque automatically** before the sweep
and **disable it again when finished** (or when stopped), so the motors actually
move and the bus is left as you found it.

## License

Apache-2.0
