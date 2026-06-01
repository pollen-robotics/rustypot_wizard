use std::time::Duration;

use anyhow::{anyhow, Result};
use rustypot::DynamixelProtocolHandler;

use crate::registers::Protocol;

pub struct Bus {
    pub dph: DynamixelProtocolHandler,
    pub port: Box<dyn serialport::SerialPort>,
}

impl Bus {
    pub fn open(port_name: &str, baud: u32, protocol: Protocol) -> Result<Self> {
        let port = serialport::new(port_name, baud)
            .timeout(Duration::from_millis(50))
            .open()
            .map_err(|e| anyhow!("Could not open {}: {}", port_name, e))?;
        let dph = match protocol {
            Protocol::V1 => DynamixelProtocolHandler::v1(),
            Protocol::V2 => DynamixelProtocolHandler::v2(),
        };
        Ok(Self { dph, port })
    }

    pub fn ping(&mut self, id: u8) -> bool {
        self.dph.ping(self.port.as_mut(), id).unwrap_or(false)
    }

    pub fn read(&mut self, id: u8, addr: u8, length: u8) -> Result<Vec<u8>> {
        self.dph
            .read(self.port.as_mut(), id, addr, length)
            .map_err(|e| anyhow!("read failed: {}", e))
    }

    pub fn write(&mut self, id: u8, addr: u8, data: &[u8]) -> Result<()> {
        self.dph
            .write(self.port.as_mut(), id, addr, data)
            .map_err(|e| anyhow!("write failed: {}", e))
    }

    pub fn reboot(&mut self, id: u8) -> Result<()> {
        self.dph
            .reboot(self.port.as_mut(), id)
            .map(|_| ())
            .map_err(|e| anyhow!("reboot failed: {}", e))
    }

    pub fn factory_reset(
        &mut self,
        id: u8,
        conserve_id_only: bool,
        conserve_id_and_baudrate: bool,
    ) -> Result<()> {
        self.dph
            .factory_reset(
                self.port.as_mut(),
                id,
                conserve_id_only,
                conserve_id_and_baudrate,
            )
            .map_err(|e| anyhow!("factory reset failed: {}", e))
    }
}

pub fn list_ports() -> Vec<String> {
    fn is_serial(base: &str) -> bool {
        // Linux: USB converters and built-in platform UARTs.
        base.starts_with("ttyACM")
            || base.starts_with("ttyUSB")
            || base.starts_with("ttyAMA")
            || base.starts_with("ttyS")
            // macOS: USB serial callout (cu.*) and dial-in (tty.*) nodes.
            // Match USB devices only to skip Bluetooth-* and other tty.* nodes.
            || base.starts_with("cu.usb")
            || base.starts_with("tty.usb")
    }

    let mut ports: Vec<String> = serialport::available_ports()
        .map(|ports| {
            ports
                .into_iter()
                .map(|p| p.port_name)
                .filter(|name| is_serial(name.rsplit('/').next().unwrap_or(name)))
                .collect()
        })
        .unwrap_or_default();

    // Fallback: libudev (which `available_ports` uses on Linux) only enumerates
    // serial devices that have an entry in the `tty` subsystem and a parent
    // device declared in udev — typically USB serial converters. Built-in
    // platform UARTs like /dev/ttyS2 on Rockchip / Allwinner SBCs aren't
    // discovered. Scan /dev directly and union the results.
    if let Ok(entries) = std::fs::read_dir("/dev") {
        for e in entries.flatten() {
            let name = e.file_name();
            let base = name.to_string_lossy();
            if is_serial(&base) {
                let full = format!("/dev/{base}");
                if !ports.contains(&full) {
                    ports.push(full);
                }
            }
        }
    }

    ports.sort();
    ports
}
