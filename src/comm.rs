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
}

pub fn list_ports() -> Vec<String> {
    serialport::available_ports()
        .map(|ports| {
            ports
                .into_iter()
                .map(|p| p.port_name)
                .filter(|name| {
                    let base = name.rsplit('/').next().unwrap_or(name);
                    base.starts_with("ttyACM")
                        || base.starts_with("ttyUSB")
                        || base.starts_with("ttyAMA")
                        || base.starts_with("ttyS")
                })
                .collect()
        })
        .unwrap_or_default()
}
