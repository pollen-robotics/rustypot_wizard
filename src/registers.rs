//! Per-model register tables.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    R,
    Rw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegType {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    Bool,
}

impl RegType {
    pub fn len(self) -> u8 {
        match self {
            RegType::U8 | RegType::I8 | RegType::Bool => 1,
            RegType::U16 | RegType::I16 => 2,
            RegType::U32 | RegType::I32 => 4,
        }
    }

    pub fn signed(self) -> bool {
        matches!(self, RegType::I8 | RegType::I16 | RegType::I32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Reg {
    pub addr: u16,
    pub name: &'static str,
    pub ty: RegType,
    pub access: Access,
}

#[derive(Debug, Clone, Copy)]
pub struct Model {
    pub name: &'static str,
    pub model_number: u16,
    pub brand: Brand,
    pub regs: &'static [Reg],
    /// Address of the model_number register (used to identify models after ping).
    pub model_number_addr: u8,
    /// Degrees per raw position count (used for display).
    pub deg_per_count: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Brand {
    Dynamixel,
    Feetech,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    V1,
    V2,
}

const fn r(addr: u16, name: &'static str, ty: RegType) -> Reg {
    Reg {
        addr,
        name,
        ty,
        access: Access::R,
    }
}
const fn rw(addr: u16, name: &'static str, ty: RegType) -> Reg {
    Reg {
        addr,
        name,
        ty,
        access: Access::Rw,
    }
}

// ---------------- Dynamixel XL330 ----------------
const XL330_REGS: &[Reg] = &[
    r(0, "Model Number", RegType::U16),
    r(2, "Model Information", RegType::U32),
    rw(6, "Firmware Version", RegType::U8),
    rw(7, "ID", RegType::U8),
    rw(8, "Baud Rate", RegType::U8),
    rw(9, "Return Delay Time", RegType::U8),
    rw(10, "Drive Mode", RegType::U8),
    rw(11, "Operating Mode", RegType::U8),
    rw(12, "Secondary(Shadow) ID", RegType::U8),
    rw(13, "Protocol Type", RegType::U8),
    rw(20, "Homing Offset", RegType::I32),
    rw(24, "Moving Threshold", RegType::U32),
    rw(31, "Temperature Limit", RegType::U8),
    rw(32, "Max Voltage Limit", RegType::U16),
    rw(34, "Min Voltage Limit", RegType::U16),
    rw(36, "PWM Limit", RegType::U16),
    rw(38, "Current Limit", RegType::U16),
    rw(40, "Acceleration Limit", RegType::U32),
    rw(44, "Velocity Limit", RegType::U32),
    rw(48, "Max Position Limit", RegType::I32),
    rw(52, "Min Position Limit", RegType::I32),
    rw(60, "Startup Configuration", RegType::U8),
    rw(62, "PWM Slope", RegType::U8),
    rw(63, "Shutdown", RegType::U8),
    rw(64, "Torque Enable", RegType::Bool),
    rw(65, "LED", RegType::U8),
    rw(68, "Status Return Level", RegType::U8),
    rw(69, "Registered Instruction", RegType::U8),
    rw(70, "Hardware Error Status", RegType::U8),
    rw(76, "Velocity I Gain", RegType::U16),
    rw(78, "Velocity P Gain", RegType::U16),
    rw(80, "Position D Gain", RegType::U16),
    rw(82, "Position I Gain", RegType::U16),
    rw(84, "Position P Gain", RegType::U16),
    rw(88, "Feedforward 2nd Gain", RegType::U16),
    rw(90, "Feedforward 1st Gain", RegType::U16),
    rw(98, "Bus Watchdog", RegType::U8),
    rw(100, "Goal PWM", RegType::U16),
    rw(102, "Goal Current", RegType::I16),
    rw(104, "Goal Velocity", RegType::I32),
    rw(108, "Profile Acceleration", RegType::U32),
    rw(112, "Profile Velocity", RegType::U32),
    rw(116, "Goal Position", RegType::I32),
    r(120, "Realtime Tick", RegType::U16),
    r(122, "Moving", RegType::U8),
    r(123, "Moving Status", RegType::U8),
    r(124, "Present PWM", RegType::U16),
    r(126, "Present Current", RegType::I16),
    r(128, "Present Velocity", RegType::I32),
    r(132, "Present Position", RegType::I32),
    r(136, "Velocity Trajectory", RegType::U32),
    r(140, "Position Trajectory", RegType::U32),
    r(144, "Present Input Voltage", RegType::U16),
    r(146, "Present Temperature", RegType::U8),
    r(147, "Backup Ready", RegType::U8),
];

// ---------------- Dynamixel XL430 ----------------
const XL430_REGS: &[Reg] = &[
    r(0, "Model Number", RegType::U16),
    r(2, "Model Information", RegType::U32),
    rw(6, "Firmware Version", RegType::U8),
    rw(7, "ID", RegType::U8),
    rw(8, "Baud Rate", RegType::U8),
    rw(9, "Return Delay Time", RegType::U8),
    rw(10, "Drive Mode", RegType::U8),
    rw(11, "Operating Mode", RegType::U8),
    rw(12, "Secondary(Shadow) ID", RegType::U8),
    rw(13, "Protocol Type", RegType::U8),
    rw(20, "Homing Offset", RegType::I32),
    rw(24, "Moving Threshold", RegType::U32),
    rw(31, "Temperature Limit", RegType::U8),
    rw(32, "Max Voltage Limit", RegType::U16),
    rw(34, "Min Voltage Limit", RegType::U16),
    rw(36, "PWM Limit", RegType::U16),
    rw(44, "Velocity Limit", RegType::U32),
    rw(48, "Max Position Limit", RegType::I32),
    rw(52, "Min Position Limit", RegType::I32),
    rw(63, "Shutdown", RegType::U8),
    rw(64, "Torque Enable", RegType::Bool),
    rw(65, "LED", RegType::U8),
    rw(68, "Status Return Level", RegType::U8),
    rw(70, "Hardware Error Status", RegType::U8),
    rw(76, "Velocity I Gain", RegType::U16),
    rw(78, "Velocity P Gain", RegType::U16),
    rw(80, "Position D Gain", RegType::U16),
    rw(82, "Position I Gain", RegType::U16),
    rw(84, "Position P Gain", RegType::U16),
    rw(88, "Feedforward 2nd Gain", RegType::U16),
    rw(90, "Feedforward 1st Gain", RegType::U16),
    rw(98, "Bus Watchdog", RegType::U8),
    rw(104, "Goal Velocity", RegType::I32),
    rw(108, "Profile Acceleration", RegType::U32),
    rw(112, "Profile Velocity", RegType::U32),
    rw(116, "Goal Position", RegType::I32),
    r(120, "Realtime Tick", RegType::U16),
    r(122, "Moving", RegType::U8),
    r(123, "Moving Status", RegType::U8),
    r(128, "Present Velocity", RegType::I32),
    r(132, "Present Position", RegType::I32),
    r(144, "Present Input Voltage", RegType::U16),
    r(146, "Present Temperature", RegType::U8),
];

// ---------------- Dynamixel MX (Protocol 2 layout) ----------------
const MX_V2_REGS: &[Reg] = &[
    r(0, "Model Number", RegType::U16),
    r(2, "Model Information", RegType::U32),
    rw(6, "Firmware Version", RegType::U8),
    rw(7, "ID", RegType::U8),
    rw(8, "Baud Rate", RegType::U8),
    rw(9, "Return Delay Time", RegType::U8),
    rw(10, "Drive Mode", RegType::U8),
    rw(11, "Operating Mode", RegType::U8),
    rw(12, "Secondary(Shadow) ID", RegType::U8),
    rw(13, "Protocol Type", RegType::U8),
    rw(20, "Homing Offset", RegType::I32),
    rw(24, "Moving Threshold", RegType::U32),
    rw(31, "Temperature Limit", RegType::U8),
    rw(32, "Max Voltage Limit", RegType::U16),
    rw(34, "Min Voltage Limit", RegType::U16),
    rw(36, "PWM Limit", RegType::U16),
    rw(38, "Current Limit", RegType::U16),
    rw(44, "Velocity Limit", RegType::U32),
    rw(48, "Max Position Limit", RegType::I32),
    rw(52, "Min Position Limit", RegType::I32),
    rw(63, "Shutdown", RegType::U8),
    rw(64, "Torque Enable", RegType::Bool),
    rw(65, "LED", RegType::U8),
    rw(68, "Status Return Level", RegType::U8),
    rw(70, "Hardware Error Status", RegType::U8),
    rw(76, "Velocity I Gain", RegType::U16),
    rw(78, "Velocity P Gain", RegType::U16),
    rw(80, "Position D Gain", RegType::U16),
    rw(82, "Position I Gain", RegType::U16),
    rw(84, "Position P Gain", RegType::U16),
    rw(102, "Goal Current", RegType::I16),
    rw(104, "Goal Velocity", RegType::I32),
    rw(108, "Profile Acceleration", RegType::U32),
    rw(112, "Profile Velocity", RegType::U32),
    rw(116, "Goal Position", RegType::I32),
    r(126, "Present Current", RegType::I16),
    r(128, "Present Velocity", RegType::I32),
    r(132, "Present Position", RegType::I32),
    r(144, "Present Input Voltage", RegType::U16),
    r(146, "Present Temperature", RegType::U8),
];

// ---------------- Dynamixel AX (Protocol 1) ----------------
const AX_REGS: &[Reg] = &[
    r(0, "Model Number", RegType::U16),
    r(2, "Firmware Version", RegType::U8),
    rw(3, "ID", RegType::U8),
    rw(4, "Baud Rate", RegType::U8),
    rw(5, "Return Delay Time", RegType::U8),
    rw(6, "CW Angle Limit", RegType::U16),
    rw(8, "CCW Angle Limit", RegType::U16),
    rw(11, "Temperature Limit", RegType::U8),
    rw(12, "Min Voltage Limit", RegType::U8),
    rw(13, "Max Voltage Limit", RegType::U8),
    rw(14, "Max Torque", RegType::U16),
    rw(16, "Status Return Level", RegType::U8),
    rw(17, "Alarm LED", RegType::U8),
    rw(18, "Shutdown", RegType::U8),
    rw(24, "Torque Enable", RegType::Bool),
    rw(25, "LED", RegType::U8),
    rw(26, "CW Compliance Margin", RegType::U8),
    rw(27, "CCW Compliance Margin", RegType::U8),
    rw(28, "CW Compliance Slope", RegType::U8),
    rw(29, "CCW Compliance Slope", RegType::U8),
    rw(30, "Goal Position", RegType::U16),
    rw(32, "Moving Speed", RegType::U16),
    rw(34, "Torque Limit", RegType::U16),
    r(36, "Present Position", RegType::U16),
    r(38, "Present Speed", RegType::U16),
    r(40, "Present Load", RegType::U16),
    r(42, "Present Voltage", RegType::U8),
    r(43, "Present Temperature", RegType::U8),
    r(44, "Registered", RegType::U8),
    r(46, "Moving", RegType::U8),
    rw(47, "Lock", RegType::U8),
    rw(48, "Punch", RegType::U16),
];

// ---------------- Dynamixel XL-320 (Protocol 2) ----------------
const XL320_REGS: &[Reg] = &[
    r(0, "Model Number", RegType::U16),
    r(2, "Firmware Version", RegType::U8),
    rw(3, "ID", RegType::U8),
    rw(4, "Baud Rate", RegType::U8),
    rw(5, "Return Delay Time", RegType::U8),
    rw(6, "CW Angle Limit", RegType::U16),
    rw(8, "CCW Angle Limit", RegType::U16),
    rw(11, "Control Mode", RegType::U8),
    rw(12, "Temperature Limit", RegType::U8),
    rw(13, "Min Voltage Limit", RegType::U8),
    rw(14, "Max Voltage Limit", RegType::U8),
    rw(15, "Max Torque", RegType::U16),
    rw(17, "Status Return Level", RegType::U8),
    rw(18, "Shutdown", RegType::U8),
    rw(24, "Torque Enable", RegType::Bool),
    rw(25, "LED", RegType::U8),
    rw(27, "D Gain", RegType::U8),
    rw(28, "I Gain", RegType::U8),
    rw(29, "P Gain", RegType::U8),
    rw(30, "Goal Position", RegType::U16),
    rw(32, "Moving Speed", RegType::U16),
    rw(35, "Torque Limit", RegType::U16),
    r(37, "Present Position", RegType::U16),
    r(39, "Present Speed", RegType::U16),
    r(41, "Present Load", RegType::U16),
    r(45, "Present Voltage", RegType::U8),
    r(46, "Present Temperature", RegType::U8),
    r(47, "Registered Instruction", RegType::U8),
    r(49, "Moving", RegType::U8),
    r(50, "Hardware Error Status", RegType::U8),
    rw(51, "Punch", RegType::U16),
];

// ---------------- Feetech STS3215 ----------------
const STS3215_REGS: &[Reg] = &[
    r(3, "Model", RegType::U16),
    rw(5, "ID", RegType::U8),
    rw(6, "Baud Rate", RegType::U8),
    rw(7, "Return Delay Time", RegType::U8),
    rw(8, "Response Status Level", RegType::U8),
    rw(9, "Min Angle Limit", RegType::I16),
    rw(11, "Max Angle Limit", RegType::I16),
    rw(13, "Max Temperature Limit", RegType::U8),
    rw(14, "Max Voltage Limit", RegType::U8),
    rw(15, "Min Voltage Limit", RegType::U8),
    rw(16, "Max Torque Limit", RegType::U16),
    rw(18, "Phase", RegType::U8),
    rw(19, "Unloading Condition", RegType::U8),
    rw(20, "LED Alarm Condition", RegType::U8),
    rw(21, "P Coefficient", RegType::U8),
    rw(22, "D Coefficient", RegType::U8),
    rw(23, "I Coefficient", RegType::U8),
    rw(24, "Minimum Startup Force", RegType::U16),
    rw(26, "CW Dead Zone", RegType::U8),
    rw(27, "CCW Dead Zone", RegType::U8),
    rw(28, "Protection Current", RegType::U16),
    rw(30, "Angular Resolution", RegType::U8),
    rw(31, "Offset", RegType::U16),
    rw(33, "Mode", RegType::U8),
    rw(34, "Protective Torque", RegType::U8),
    rw(35, "Protection Time", RegType::U8),
    rw(36, "Overload Torque", RegType::U8),
    rw(37, "Speed Closed Loop P", RegType::U8),
    rw(38, "Over Current Protection Time", RegType::U8),
    rw(39, "Velocity Closed Loop I", RegType::U8),
    rw(40, "Torque Enable", RegType::Bool),
    rw(41, "Acceleration", RegType::U8),
    rw(42, "Goal Position", RegType::I16),
    rw(44, "Goal Time", RegType::U16),
    rw(46, "Goal Speed", RegType::U16),
    rw(48, "Torque Limit", RegType::U16),
    rw(55, "Lock", RegType::Bool),
    r(56, "Present Position", RegType::I16),
    r(58, "Present Speed", RegType::U16),
    r(60, "Present Load", RegType::U16),
    r(62, "Present Voltage", RegType::U8),
    r(63, "Present Temperature", RegType::U8),
    r(65, "Status", RegType::U8),
    r(66, "Moving", RegType::Bool),
    r(69, "Present Current", RegType::U16),
    rw(85, "Maximum Acceleration", RegType::U16),
];

// ---------------- Feetech SCS0009 ----------------
const SCS0009_REGS: &[Reg] = &[
    r(3, "Model", RegType::U16),
    rw(5, "ID", RegType::U8),
    rw(6, "Baud Rate", RegType::U8),
    rw(7, "Return Delay Time", RegType::U8),
    rw(8, "Response Status Level", RegType::U8),
    rw(9, "Min Angle Limit", RegType::U16),
    rw(11, "Max Angle Limit", RegType::U16),
    rw(13, "Max Temperature Limit", RegType::U8),
    rw(14, "Max Voltage Limit", RegType::U8),
    rw(15, "Min Voltage Limit", RegType::U8),
    rw(16, "Max Torque Limit", RegType::U16),
    rw(18, "Phase", RegType::U8),
    rw(19, "Unloading Condition", RegType::U8),
    rw(20, "LED Alarm Condition", RegType::U8),
    rw(21, "P Coefficient", RegType::U8),
    rw(22, "D Coefficient", RegType::U8),
    rw(23, "I Coefficient", RegType::U8),
    rw(24, "Minimum Startup Force", RegType::U16),
    rw(26, "CW Dead Zone", RegType::U8),
    rw(27, "CCW Dead Zone", RegType::U8),
    rw(28, "Protection Current", RegType::U16),
    rw(40, "Torque Enable", RegType::Bool),
    rw(42, "Goal Position", RegType::U16),
    rw(46, "Goal Speed", RegType::U16),
    r(56, "Present Position", RegType::U16),
    r(58, "Present Speed", RegType::U16),
    r(60, "Present Load", RegType::U16),
    r(62, "Present Voltage", RegType::U8),
    r(63, "Present Temperature", RegType::U8),
    r(66, "Moving", RegType::Bool),
];

pub const MODELS: &[Model] = &[
    Model {
        name: "XL330-M077",
        model_number: 1190,
        brand: Brand::Dynamixel,
        regs: XL330_REGS,
        model_number_addr: 0,
        deg_per_count: 360.0 / 4096.0,
    },
    Model {
        name: "XL330-M288",
        model_number: 1200,
        brand: Brand::Dynamixel,
        regs: XL330_REGS,
        model_number_addr: 0,
        deg_per_count: 360.0 / 4096.0,
    },
    Model {
        name: "XL430-W250",
        model_number: 1060,
        brand: Brand::Dynamixel,
        regs: XL430_REGS,
        model_number_addr: 0,
        deg_per_count: 360.0 / 4096.0,
    },
    Model {
        name: "XL430-W250-2",
        model_number: 1090,
        brand: Brand::Dynamixel,
        regs: XL430_REGS,
        model_number_addr: 0,
        deg_per_count: 360.0 / 4096.0,
    },
    Model {
        name: "MX-28 (v2)",
        model_number: 30,
        brand: Brand::Dynamixel,
        regs: MX_V2_REGS,
        model_number_addr: 0,
        deg_per_count: 360.0 / 4096.0,
    },
    Model {
        name: "MX-64 (v2)",
        model_number: 311,
        brand: Brand::Dynamixel,
        regs: MX_V2_REGS,
        model_number_addr: 0,
        deg_per_count: 360.0 / 4096.0,
    },
    Model {
        name: "MX-106 (v2)",
        model_number: 321,
        brand: Brand::Dynamixel,
        regs: MX_V2_REGS,
        model_number_addr: 0,
        deg_per_count: 360.0 / 4096.0,
    },
    Model {
        name: "AX-12",
        model_number: 12,
        brand: Brand::Dynamixel,
        regs: AX_REGS,
        model_number_addr: 0,
        deg_per_count: 300.0 / 1024.0,
    },
    Model {
        name: "AX-12W",
        model_number: 300,
        brand: Brand::Dynamixel,
        regs: AX_REGS,
        model_number_addr: 0,
        deg_per_count: 300.0 / 1024.0,
    },
    Model {
        name: "AX-18A",
        model_number: 18,
        brand: Brand::Dynamixel,
        regs: AX_REGS,
        model_number_addr: 0,
        deg_per_count: 300.0 / 1024.0,
    },
    Model {
        name: "XL-320",
        model_number: 350,
        brand: Brand::Dynamixel,
        regs: XL320_REGS,
        model_number_addr: 0,
        deg_per_count: 300.0 / 1024.0,
    },
    Model {
        name: "STS3215",
        model_number: 2307,
        brand: Brand::Feetech,
        regs: STS3215_REGS,
        model_number_addr: 3,
        deg_per_count: 360.0 / 4096.0,
    },
    Model {
        name: "SCS0009",
        model_number: 1280,
        brand: Brand::Feetech,
        regs: SCS0009_REGS,
        model_number_addr: 3,
        deg_per_count: 300.0 / 1024.0,
    },
];

pub fn lookup_model(brand: Brand, model_number: u16) -> Option<&'static Model> {
    MODELS
        .iter()
        .find(|m| m.brand == brand && m.model_number == model_number)
}

/// Default register table to display when the model is unknown.
pub fn default_regs(brand: Brand) -> &'static [Reg] {
    match brand {
        Brand::Dynamixel => XL330_REGS,
        Brand::Feetech => STS3215_REGS,
    }
}

pub fn model_number_addr(brand: Brand) -> u8 {
    match brand {
        Brand::Dynamixel => 0,
        Brand::Feetech => 3,
    }
}

pub const COMMON_BAUDRATES: &[u32] = &[
    9600, 57600, 115200, 1_000_000, 2_000_000, 3_000_000, 4_000_000, 4_500_000, 6_000_000,
];

pub fn decode_value(bytes: &[u8], ty: RegType) -> i64 {
    match ty {
        RegType::U8 | RegType::Bool => bytes[0] as i64,
        RegType::I8 => bytes[0] as i8 as i64,
        RegType::U16 => u16::from_le_bytes([bytes[0], bytes[1]]) as i64,
        RegType::I16 => i16::from_le_bytes([bytes[0], bytes[1]]) as i64,
        RegType::U32 => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as i64,
        RegType::I32 => i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as i64,
    }
}

/// Map "standard" capability names → concrete `Reg` for the current model.
/// Values are looked up by register name with a small alias list.
#[derive(Debug, Default, Clone, Copy)]
pub struct MotorControl {
    pub torque_enable: Option<Reg>,
    pub goal_position: Option<Reg>,
    pub present_position: Option<Reg>,
    pub present_velocity: Option<Reg>,
    pub present_current: Option<Reg>,
    pub present_load: Option<Reg>,
    pub present_voltage: Option<Reg>,
    pub present_temperature: Option<Reg>,
    pub min_position: Option<Reg>,
    pub max_position: Option<Reg>,
    pub moving: Option<Reg>,
    pub led: Option<Reg>,
}

fn find<'a>(regs: &'a [Reg], names: &[&str]) -> Option<Reg> {
    for n in names {
        if let Some(r) = regs.iter().find(|r| r.name == *n) {
            return Some(*r);
        }
    }
    None
}

impl MotorControl {
    pub fn from_regs(regs: &[Reg]) -> Self {
        Self {
            torque_enable: find(regs, &["Torque Enable"]),
            goal_position: find(regs, &["Goal Position"]),
            present_position: find(regs, &["Present Position"]),
            present_velocity: find(regs, &["Present Velocity", "Present Speed"]),
            present_current: find(regs, &["Present Current"]),
            present_load: find(regs, &["Present Load"]),
            present_voltage: find(regs, &["Present Input Voltage", "Present Voltage"]),
            present_temperature: find(regs, &["Present Temperature"]),
            min_position: find(
                regs,
                &["Min Position Limit", "Min Angle Limit", "CW Angle Limit"],
            ),
            max_position: find(
                regs,
                &["Max Position Limit", "Max Angle Limit", "CCW Angle Limit"],
            ),
            moving: find(regs, &["Moving"]),
            led: find(regs, &["LED"]),
        }
    }

    /// Registers polled in the live-refresh loop.
    pub fn live_regs(self) -> Vec<Reg> {
        [
            self.torque_enable,
            self.present_position,
            self.present_velocity,
            self.present_current,
            self.present_load,
            self.present_voltage,
            self.present_temperature,
            self.moving,
            self.goal_position,
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

pub fn encode_value(value: i64, ty: RegType) -> Vec<u8> {
    match ty {
        RegType::U8 | RegType::Bool => vec![value as u8],
        RegType::I8 => vec![(value as i8) as u8],
        RegType::U16 => (value as u16).to_le_bytes().to_vec(),
        RegType::I16 => (value as i16).to_le_bytes().to_vec(),
        RegType::U32 => (value as u32).to_le_bytes().to_vec(),
        RegType::I32 => (value as i32).to_le_bytes().to_vec(),
    }
}
