use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

use ratatui::layout::Rect;

use crate::bench::{BenchIo, BenchKind, BenchResult, BenchRun, Stats};
use crate::comm::{list_ports, Bus};
use crate::config::{load_last_port, save_last_port};
use crate::registers::{
    decode_value, default_regs, encode_value, lookup_model, model_number_addr, Brand, Model,
    MotorControl, Protocol, Reg, COMMON_BAUDRATES,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Setup,
    Main,
    Bench,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupField {
    Brand,
    Port,
    Baud,
    Protocol,
    ScanRange,
}

const SETUP_FIELDS: &[SetupField] = &[
    SetupField::Brand,
    SetupField::Port,
    SetupField::Baud,
    SetupField::Protocol,
    SetupField::ScanRange,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    Motors,
    Registers,
}

pub struct DiscoveredMotor {
    pub id: u8,
    pub model_number: Option<u16>,
    pub model: Option<&'static Model>,
}

impl DiscoveredMotor {
    pub fn display(&self) -> String {
        match self.model {
            Some(m) => format!("[ID:{:>3}] {}", self.id, m.name),
            None => match self.model_number {
                Some(mn) => format!("[ID:{:>3}] Unknown (mn={})", self.id, mn),
                None => format!("[ID:{:>3}] Unknown", self.id),
            },
        }
    }
}

pub struct App {
    pub mode: Mode,

    // Setup state
    pub setup_focus: usize,
    pub brand: Brand,
    pub ports: Vec<String>,
    pub port_idx: usize,
    pub baud_idx: usize,
    pub protocol: Protocol,
    pub scan_max: u8,

    // Connected state
    pub bus: Option<Bus>,
    pub motors: Vec<DiscoveredMotor>,
    pub motor_idx: usize,
    pub focus: FocusedPane,
    pub reg_idx: usize,
    /// addr -> raw bytes (Ok) or error string
    pub reg_values: HashMap<u16, std::result::Result<Vec<u8>, String>>,

    pub status: String,
    pub editing: Option<EditState>,
    pub should_quit: bool,
    pub scan: Option<ScanProgress>,
    pub last_live: Option<Instant>,
    pub hits: RefCell<Vec<HitZone>>,
    pub confirm: Option<ConfirmAction>,

    // Benchmark state
    pub bench_idx: usize,
    pub bench_secs: f64,
    pub bench_amp_deg: f64,
    pub bench_freq_hz: f64,
    pub bench_run: Option<BenchRun>,
    pub bench_results: HashMap<BenchKindKey, BenchResult>,
    /// Pending benchmarks to run back-to-back (populated by "run all").
    pub bench_queue: Vec<BenchKind>,
}

/// `BenchKind` is not `Hash`; use its discriminant as a stable map key.
pub type BenchKindKey = usize;

pub fn bench_key(kind: BenchKind) -> BenchKindKey {
    BenchKind::ALL.iter().position(|k| *k == kind).unwrap_or(0)
}

#[derive(Debug, Clone, Copy)]
pub enum ConfirmAction {
    FactoryReset,
}

#[derive(Debug, Clone, Copy)]
pub struct HitZone {
    pub rect: Rect,
    pub hit: Hit,
}

#[derive(Debug, Clone, Copy)]
pub enum Hit {
    SetupField(usize),
    Connect,
    BackToSetup,
    MotorIdx(usize),
    RegIdx(usize),
    StartOrStopScan,
    ToggleTorque,
    /// Click on the goal slider — value chosen by the x ratio inside the rect.
    GoalSlider,
    EditGoal,
    Reboot,
    FactoryReset,
    ConfirmYes,
    ConfirmNo,
    BenchIdx(usize),
    RunBench,
    RunAllBench,
}

pub struct ScanProgress {
    pub next_id: u16,
    pub max: u8,
}

impl ScanProgress {
    pub fn ratio(&self) -> f64 {
        let total = self.max as f64 + 1.0;
        (self.next_id as f64 / total).clamp(0.0, 1.0)
    }
    pub fn done(&self) -> bool {
        self.next_id > self.max as u16
    }
}

pub struct EditState {
    pub addr: u16,
    pub buffer: String,
}

impl App {
    pub fn new() -> Self {
        let ports = list_ports();
        let port_idx = load_last_port()
            .and_then(|saved| ports.iter().position(|p| p == &saved))
            .unwrap_or(0);
        let baud_idx = COMMON_BAUDRATES
            .iter()
            .position(|&b| b == 1_000_000)
            .unwrap_or(0);
        Self {
            mode: Mode::Setup,
            setup_focus: 0,
            brand: Brand::Dynamixel,
            ports,
            port_idx,
            baud_idx,
            protocol: Protocol::V2,
            scan_max: 255,
            bus: None,
            motors: Vec::new(),
            motor_idx: 0,
            focus: FocusedPane::Motors,
            reg_idx: 0,
            reg_values: HashMap::new(),
            status: "Configure connection then press Enter to scan.".to_string(),
            editing: None,
            should_quit: false,
            scan: None,
            last_live: None,
            hits: RefCell::new(Vec::new()),
            confirm: None,
            bench_idx: 0,
            bench_secs: 10.0,
            bench_amp_deg: 5.0,
            bench_freq_hz: 0.5,
            bench_run: None,
            bench_results: HashMap::new(),
            bench_queue: Vec::new(),
        }
    }

    pub fn current_port(&self) -> Option<&str> {
        self.ports.get(self.port_idx).map(|s| s.as_str())
    }

    pub fn current_baud(&self) -> u32 {
        COMMON_BAUDRATES[self.baud_idx]
    }

    pub fn refresh_ports(&mut self) {
        self.ports = list_ports();
        if self.port_idx >= self.ports.len() {
            self.port_idx = self.ports.len().saturating_sub(1);
        }
    }

    pub fn current_field(&self) -> SetupField {
        SETUP_FIELDS[self.setup_focus]
    }

    pub fn cycle_field(&mut self, forward: bool) {
        let n = SETUP_FIELDS.len();
        self.setup_focus = if forward {
            (self.setup_focus + 1) % n
        } else {
            (self.setup_focus + n - 1) % n
        };
    }

    pub fn adjust_field(&mut self, delta: i32) {
        match self.current_field() {
            SetupField::Brand => {
                self.brand = match self.brand {
                    Brand::Dynamixel => Brand::Feetech,
                    Brand::Feetech => Brand::Dynamixel,
                };
            }
            SetupField::Port => {
                if !self.ports.is_empty() {
                    let n = self.ports.len() as i32;
                    let i = (self.port_idx as i32 + delta).rem_euclid(n);
                    self.port_idx = i as usize;
                }
            }
            SetupField::Baud => {
                let n = COMMON_BAUDRATES.len() as i32;
                let i = (self.baud_idx as i32 + delta).rem_euclid(n);
                self.baud_idx = i as usize;
            }
            SetupField::Protocol => {
                self.protocol = match self.protocol {
                    Protocol::V1 => Protocol::V2,
                    Protocol::V2 => Protocol::V1,
                };
            }
            SetupField::ScanRange => {
                let v = self.scan_max as i32 + delta;
                self.scan_max = v.clamp(1, 255) as u8;
            }
        }
    }

    pub fn connect_and_scan(&mut self) {
        let port = match self.current_port() {
            Some(p) => p.to_string(),
            None => {
                self.status = "No serial port available. Plug a device and press F5.".into();
                return;
            }
        };
        let baud = self.current_baud();
        let protocol = self.protocol;
        match Bus::open(&port, baud, protocol) {
            Ok(bus) => {
                self.bus = Some(bus);
                save_last_port(&port);
                self.status = format!("Opened {} @ {} bps. Scanning…", port, baud);
                self.start_scan();
                self.mode = Mode::Main;
            }
            Err(e) => {
                self.status = format!("Open error: {}", e);
            }
        }
    }

    pub fn start_scan(&mut self) {
        self.motors.clear();
        self.motor_idx = 0;
        self.reg_idx = 0;
        self.reg_values.clear();
        self.scan = Some(ScanProgress {
            next_id: 0,
            max: self.scan_max,
        });
        self.status = format!("Scanning IDs 0..={}…", self.scan_max);
    }

    /// Ping one id and advance the scan. Returns true if scan is still running.
    pub fn tick_scan(&mut self) -> bool {
        let Some(scan) = self.scan.as_mut() else {
            return false;
        };
        if scan.done() {
            self.finish_scan();
            return false;
        }
        let id = scan.next_id as u8;
        scan.next_id += 1;
        if let Some(bus) = self.bus.as_mut() {
            if bus.ping(id) {
                let mn_addr = model_number_addr(self.brand);
                let model_number = bus
                    .read(id, mn_addr, 2)
                    .ok()
                    .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]));
                let model = model_number.and_then(|mn| lookup_model(self.brand, mn));
                let was_empty = self.motors.is_empty();
                self.motors.push(DiscoveredMotor {
                    id,
                    model_number,
                    model,
                });
                if was_empty {
                    self.read_all_regs_for_selected();
                }
            }
        }
        if self.scan.as_ref().map(|s| s.done()).unwrap_or(true) {
            self.finish_scan();
            return false;
        }
        true
    }

    pub fn stop_scan(&mut self) {
        if self.scan.is_none() {
            return;
        }
        self.finish_scan();
        self.status = format!(
            "Scan stopped — {} motor(s) found.",
            self.motors.len()
        );
    }

    fn finish_scan(&mut self) {
        self.scan = None;
        self.status = format!("Found {} motor(s).", self.motors.len());
        if !self.motors.is_empty() && self.reg_values.is_empty() {
            self.read_all_regs_for_selected();
        }
    }

    pub fn selected_motor(&self) -> Option<&DiscoveredMotor> {
        self.motors.get(self.motor_idx)
    }

    pub fn current_regs(&self) -> &'static [Reg] {
        match self.selected_motor().and_then(|m| m.model) {
            Some(m) => m.regs,
            None => default_regs(self.brand),
        }
    }

    pub fn read_all_regs_for_selected(&mut self) {
        self.reg_values.clear();
        let Some(motor) = self.selected_motor() else {
            return;
        };
        let id = motor.id;
        let regs: Vec<Reg> = self.current_regs().to_vec();
        let Some(bus) = self.bus.as_mut() else {
            return;
        };
        for reg in regs {
            // Addresses fit in u8 for these protocols/models.
            let addr = reg.addr as u8;
            let res = bus.read(id, addr, reg.ty.len());
            self.reg_values.insert(
                reg.addr,
                res.map_err(|e| e.to_string()),
            );
        }
    }

    pub fn read_selected_reg(&mut self) {
        let Some(motor) = self.selected_motor() else {
            return;
        };
        let id = motor.id;
        let regs = self.current_regs();
        let Some(reg) = regs.get(self.reg_idx).copied() else {
            return;
        };
        let Some(bus) = self.bus.as_mut() else { return };
        let addr = reg.addr as u8;
        let res = bus.read(id, addr, reg.ty.len()).map_err(|e| e.to_string());
        self.reg_values.insert(reg.addr, res);
    }

    pub fn move_motor(&mut self, delta: i32) {
        if self.motors.is_empty() {
            return;
        }
        let n = self.motors.len() as i32;
        self.motor_idx = (self.motor_idx as i32 + delta).rem_euclid(n) as usize;
        self.reg_idx = 0;
        self.read_all_regs_for_selected();
    }

    pub fn move_reg(&mut self, delta: i32) {
        let regs = self.current_regs();
        if regs.is_empty() {
            return;
        }
        let n = regs.len() as i32;
        self.reg_idx = (self.reg_idx as i32 + delta).rem_euclid(n) as usize;
    }

    pub fn start_edit(&mut self) {
        let regs = self.current_regs();
        let Some(reg) = regs.get(self.reg_idx).copied() else {
            return;
        };
        if reg.access != crate::registers::Access::Rw {
            self.status = "Register is read-only.".into();
            return;
        }
        let buffer = match self.reg_values.get(&reg.addr).and_then(|r| r.as_ref().ok()) {
            Some(bytes) => decode_value(bytes, reg.ty).to_string(),
            None => String::new(),
        };
        self.editing = Some(EditState {
            addr: reg.addr,
            buffer,
        });
        self.status = "Editing — Enter to commit, Esc to cancel.".into();
    }

    pub fn commit_edit(&mut self) {
        let Some(edit) = self.editing.take() else {
            return;
        };
        let Some(motor) = self.selected_motor() else {
            return;
        };
        let id = motor.id;
        let reg = match self.current_regs().iter().find(|r| r.addr == edit.addr).copied() {
            Some(r) => r,
            None => return,
        };
        let value: i64 = match edit.buffer.trim().parse() {
            Ok(v) => v,
            Err(_) => {
                self.status = format!("Invalid number: '{}'", edit.buffer);
                return;
            }
        };
        let bytes = encode_value(value, reg.ty);
        let Some(bus) = self.bus.as_mut() else { return };
        match bus.write(id, reg.addr as u8, &bytes) {
            Ok(()) => {
                self.status = format!("Wrote {} to {} (id {}).", value, reg.name, id);
                self.read_selected_reg();
            }
            Err(e) => {
                self.status = format!("Write failed: {}", e);
            }
        }
    }

    pub fn cancel_edit(&mut self) {
        self.editing = None;
        self.status = "Edit cancelled.".into();
    }

    pub fn motor_control(&self) -> MotorControl {
        MotorControl::from_regs(self.current_regs())
    }

    /// Read raw register value from the cache.
    pub fn cached(&self, reg: Reg) -> Option<i64> {
        self.reg_values
            .get(&reg.addr)
            .and_then(|r| r.as_ref().ok())
            .map(|bytes| decode_value(bytes, reg.ty))
    }

    pub fn toggle_torque(&mut self) {
        let ctl = self.motor_control();
        let Some(reg) = ctl.torque_enable else {
            self.status = "No torque_enable register on this model.".into();
            return;
        };
        let Some(motor) = self.selected_motor() else {
            return;
        };
        let id = motor.id;
        let current = self.cached(reg).unwrap_or(0);
        let new_val = if current == 0 { 1 } else { 0 };
        let bytes = encode_value(new_val, reg.ty);
        let Some(bus) = self.bus.as_mut() else { return };
        match bus.write(id, reg.addr as u8, &bytes) {
            Ok(()) => {
                self.reg_values.insert(reg.addr, Ok(bytes));
                self.status = format!(
                    "Torque {} on id {}.",
                    if new_val == 1 { "enabled" } else { "disabled" },
                    id
                );
            }
            Err(e) => self.status = format!("Torque write failed: {}", e),
        }
    }

    pub fn toggle_led(&mut self) {
        let ctl = self.motor_control();
        let Some(reg) = ctl.led else {
            self.status = "No LED register on this model.".into();
            return;
        };
        let Some(motor) = self.selected_motor() else {
            return;
        };
        let id = motor.id;
        let current = self.cached(reg).unwrap_or(0);
        let new_val = if current == 0 { 1 } else { 0 };
        let bytes = encode_value(new_val, reg.ty);
        let Some(bus) = self.bus.as_mut() else { return };
        if let Err(e) = bus.write(id, reg.addr as u8, &bytes) {
            self.status = format!("LED write failed: {}", e);
        } else {
            self.reg_values.insert(reg.addr, Ok(bytes));
        }
    }

    pub fn position_bounds(&self) -> Option<(i64, i64)> {
        let ctl = self.motor_control();
        let lo = ctl.min_position.and_then(|r| self.cached(r));
        let hi = ctl.max_position.and_then(|r| self.cached(r));
        match (lo, hi) {
            (Some(a), Some(b)) if a < b => Some((a, b)),
            _ => {
                // Fallback based on goal_position type.
                let reg = ctl.goal_position?;
                let (lo, hi) = match reg.ty {
                    crate::registers::RegType::U16 => (0, 4095),
                    crate::registers::RegType::I16 => (-2048, 2047),
                    crate::registers::RegType::I32 => (0, 4095),
                    _ => (0, 4095),
                };
                Some((lo, hi))
            }
        }
    }

    pub fn nudge_goal(&mut self, delta: i64) {
        let ctl = self.motor_control();
        let Some(reg) = ctl.goal_position else {
            self.status = "No goal_position register on this model.".into();
            return;
        };
        let Some(motor) = self.selected_motor() else {
            return;
        };
        let id = motor.id;
        let current = self
            .cached(reg)
            .or_else(|| ctl.present_position.and_then(|r| self.cached(r)))
            .unwrap_or(0);
        let mut value = current + delta;
        if let Some((lo, hi)) = self.position_bounds() {
            value = value.clamp(lo, hi);
        }
        let bytes = encode_value(value, reg.ty);
        let Some(bus) = self.bus.as_mut() else { return };
        match bus.write(id, reg.addr as u8, &bytes) {
            Ok(()) => {
                self.reg_values.insert(reg.addr, Ok(bytes));
                self.status = format!("Goal Position → {} (id {}).", value, id);
            }
            Err(e) => self.status = format!("Goal write failed: {}", e),
        }
    }

    pub fn start_edit_goal(&mut self) {
        let ctl = self.motor_control();
        let Some(reg) = ctl.goal_position else {
            self.status = "No goal_position register on this model.".into();
            return;
        };
        // Position the register cursor on goal_position so editing logic reuses
        // the existing register-edit path.
        if let Some(idx) = self
            .current_regs()
            .iter()
            .position(|r| r.addr == reg.addr)
        {
            self.reg_idx = idx;
        }
        self.start_edit();
    }

    pub fn set_goal_ratio(&mut self, ratio: f64) {
        let ctl = self.motor_control();
        let Some(reg) = ctl.goal_position else {
            return;
        };
        let Some(motor) = self.selected_motor() else {
            return;
        };
        let id = motor.id;
        let (lo, hi) = self.position_bounds().unwrap_or((0, 4095));
        let span = (hi - lo) as f64;
        let value = lo + (ratio.clamp(0.0, 1.0) * span).round() as i64;
        let bytes = encode_value(value, reg.ty);
        let Some(bus) = self.bus.as_mut() else { return };
        match bus.write(id, reg.addr as u8, &bytes) {
            Ok(()) => {
                self.reg_values.insert(reg.addr, Ok(bytes));
                self.status = format!("Goal Position → {} (id {}).", value, id);
            }
            Err(e) => self.status = format!("Goal write failed: {}", e),
        }
    }

    /// Map a click in screen coordinates to a Hit on the registered zones.
    /// Returns the smallest matching zone (so widgets nested inside others win).
    pub fn hit_at(&self, x: u16, y: u16) -> Option<HitZone> {
        let zones = self.hits.borrow();
        let mut best: Option<HitZone> = None;
        for z in zones.iter() {
            if x >= z.rect.x
                && x < z.rect.x + z.rect.width
                && y >= z.rect.y
                && y < z.rect.y + z.rect.height
            {
                let area = z.rect.width as u32 * z.rect.height as u32;
                let ba = best
                    .map(|b| b.rect.width as u32 * b.rect.height as u32)
                    .unwrap_or(u32::MAX);
                if area < ba {
                    best = Some(*z);
                }
            }
        }
        best
    }

    pub fn handle_click(&mut self, x: u16, y: u16) {
        let Some(zone) = self.hit_at(x, y) else { return };
        match zone.hit {
            Hit::SetupField(i) => {
                if i == self.setup_focus {
                    self.adjust_field(1);
                } else {
                    self.setup_focus = i;
                }
            }
            Hit::Connect => self.connect_and_scan(),
            Hit::BackToSetup => {
                self.mode = Mode::Setup;
                self.bus = None;
                self.status = "Disconnected.".into();
            }
            Hit::MotorIdx(i) => {
                if i < self.motors.len() {
                    self.motor_idx = i;
                    self.focus = FocusedPane::Motors;
                    self.read_all_regs_for_selected();
                }
            }
            Hit::RegIdx(i) => {
                let n = self.current_regs().len();
                if i < n {
                    self.reg_idx = i;
                    self.focus = FocusedPane::Registers;
                }
            }
            Hit::StartOrStopScan => {
                if self.scan.is_some() {
                    self.stop_scan();
                } else {
                    self.start_scan();
                }
            }
            Hit::ToggleTorque => self.toggle_torque(),
            Hit::GoalSlider => {
                let span = zone.rect.width.saturating_sub(1).max(1) as f64;
                let dx = x.saturating_sub(zone.rect.x) as f64;
                self.set_goal_ratio(dx / span);
            }
            Hit::EditGoal => self.start_edit_goal(),
            Hit::Reboot => self.reboot_selected(),
            Hit::FactoryReset => self.request_factory_reset(),
            Hit::ConfirmYes => self.commit_confirm(),
            Hit::ConfirmNo => self.cancel_confirm(),
            Hit::BenchIdx(i) => {
                if i < BenchKind::ALL.len() {
                    self.bench_idx = i;
                }
            }
            Hit::RunBench => self.start_selected_bench(),
            Hit::RunAllBench => self.run_all_bench(),
        }
    }

    pub fn reboot_selected(&mut self) {
        let Some(motor) = self.selected_motor() else {
            return;
        };
        let id = motor.id;
        let Some(bus) = self.bus.as_mut() else { return };
        match bus.reboot(id) {
            Ok(()) => {
                self.status = format!("Reboot sent to id {}.", id);
                self.reg_values.clear();
            }
            Err(e) => self.status = format!("Reboot failed: {}", e),
        }
    }

    pub fn request_factory_reset(&mut self) {
        if self.selected_motor().is_none() {
            return;
        }
        self.confirm = Some(ConfirmAction::FactoryReset);
        self.status = "Factory reset?  Enter/y confirm, Esc/n cancel.".into();
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm = None;
        self.status = "Cancelled.".into();
    }

    pub fn commit_confirm(&mut self) {
        let Some(action) = self.confirm.take() else {
            return;
        };
        match action {
            ConfirmAction::FactoryReset => self.do_factory_reset(),
        }
    }

    fn do_factory_reset(&mut self) {
        let Some(motor) = self.selected_motor() else {
            return;
        };
        let id = motor.id;
        // For V2 keep ID + baud so we stay in contact. For V1 the protocol
        // does not support conservation flags — pass false/false.
        let (conserve_id, conserve_id_and_baud) = match self.protocol {
            Protocol::V1 => (false, false),
            Protocol::V2 => (false, true),
        };
        let Some(bus) = self.bus.as_mut() else { return };
        match bus.factory_reset(id, conserve_id, conserve_id_and_baud) {
            Ok(()) => {
                self.status = format!("Factory reset sent to id {}.", id);
                self.reg_values.clear();
            }
            Err(e) => self.status = format!("Factory reset failed: {}", e),
        }
    }

    /// Refresh the small "live" register set for the selected motor.
    /// Called from the main loop on a timer; values are written into
    /// `reg_values` so both the right panel and the register table see them.
    pub fn tick_live(&mut self) {
        if self.scan.is_some()
            || self.editing.is_some()
            || self.confirm.is_some()
            || self.bus.is_none()
        {
            return;
        }
        let now = Instant::now();
        if let Some(t) = self.last_live {
            if now.duration_since(t).as_millis() < 150 {
                return;
            }
        }
        self.last_live = Some(now);

        let Some(motor) = self.selected_motor() else {
            return;
        };
        let id = motor.id;
        let regs = self.motor_control().live_regs();
        let bus = self.bus.as_mut().unwrap();
        for reg in regs {
            let res = bus
                .read(id, reg.addr as u8, reg.ty.len())
                .map_err(|e| e.to_string());
            self.reg_values.insert(reg.addr, res);
        }
    }

    // ---------------- Benchmarks ----------------

    pub fn open_bench(&mut self) {
        if self.motors.is_empty() {
            self.status = "Scan for motors before benchmarking (press s).".into();
            return;
        }
        if self.scan.is_some() {
            self.stop_scan();
        }
        self.mode = Mode::Bench;
        self.status = "Benchmark — ↑↓ select, Enter run, a run all, Esc back.".into();
    }

    pub fn selected_bench(&self) -> BenchKind {
        BenchKind::ALL[self.bench_idx.min(BenchKind::ALL.len() - 1)]
    }

    pub fn move_bench(&mut self, delta: i32) {
        let n = BenchKind::ALL.len() as i32;
        self.bench_idx = (self.bench_idx as i32 + delta).rem_euclid(n) as usize;
    }

    pub fn adjust_secs(&mut self, up: bool) {
        // Step in a friendly progression of run durations (seconds).
        const STEPS: &[f64] = &[1.0, 2.0, 5.0, 10.0, 20.0, 30.0, 60.0, 120.0];
        let cur = self.bench_secs;
        if up {
            self.bench_secs = STEPS
                .iter()
                .copied()
                .find(|&s| s > cur + 0.001)
                .unwrap_or(120.0);
        } else {
            self.bench_secs = STEPS
                .iter()
                .rev()
                .copied()
                .find(|&s| s < cur - 0.001)
                .unwrap_or(1.0);
        }
    }

    pub fn adjust_amp(&mut self, delta: f64) {
        self.bench_amp_deg = (self.bench_amp_deg + delta).clamp(0.0, 45.0);
    }

    /// Resolve register addresses from the selected motor's model (homogeneous-bus assumption).
    fn bench_io(&self) -> BenchIo {
        let ctl = self.motor_control();
        let deg_per_count = self
            .selected_motor()
            .and_then(|m| m.model)
            .map(|m| m.deg_per_count)
            .unwrap_or(360.0 / 4096.0);
        let (pos_addr, pos_ty, has_pos) = match ctl.present_position {
            Some(r) => (r.addr as u8, r.ty, true),
            None => (0, crate::registers::RegType::I32, false),
        };
        let (goal_addr, goal_ty, has_goal) = match ctl.goal_position {
            Some(r) => (r.addr as u8, r.ty, true),
            None => (0, crate::registers::RegType::I32, false),
        };
        let (torque_addr, torque_ty, has_torque) = match ctl.torque_enable {
            Some(r) => (r.addr as u8, r.ty, true),
            None => (0, crate::registers::RegType::Bool, false),
        };
        BenchIo {
            pos_addr,
            pos_ty,
            has_pos,
            goal_addr,
            goal_ty,
            has_goal,
            torque_addr,
            torque_ty,
            has_torque,
            deg_per_count,
        }
    }

    /// Write the torque-enable register on every involved motor (sine runs only).
    fn set_torque(&mut self, ids: &[u8], io: BenchIo, on: bool) {
        if !io.has_torque {
            return;
        }
        let bytes = encode_value(on as i64, io.torque_ty);
        if let Some(bus) = self.bus.as_mut() {
            for &id in ids {
                let _ = bus.write(id, io.torque_addr, &bytes);
            }
        }
    }

    pub fn run_all_bench(&mut self) {
        if self.bench_run.is_some() {
            return;
        }
        self.bench_queue = BenchKind::ALL.to_vec();
        let first = self.bench_queue.remove(0);
        self.start_bench(first);
    }

    pub fn start_selected_bench(&mut self) {
        if self.bench_run.is_some() {
            return;
        }
        self.bench_queue.clear();
        let kind = self.selected_bench();
        self.start_bench(kind);
    }

    fn start_bench(&mut self, kind: BenchKind) {
        let io = self.bench_io();
        // Every benchmark except the pings needs a Present Position register.
        if !io.has_pos && !matches!(kind, BenchKind::PingOne | BenchKind::PingAll) {
            self.status = "No Present Position register on this model.".into();
            self.bench_queue.clear();
            return;
        }
        if kind.writes() && !io.has_goal {
            self.status = "No Goal Position register on this model.".into();
            self.bench_queue.clear();
            return;
        }

        let ids: Vec<u8> = if kind.uses_all() {
            self.motors.iter().map(|m| m.id).collect()
        } else {
            match self.selected_motor() {
                Some(m) => vec![m.id],
                None => {
                    self.status = "No motor selected.".into();
                    return;
                }
            }
        };

        // Capture each motor's resting position once: it is the goal for "hold"
        // runs and the sine centre for "sine" runs, and is restored on finish.
        // Abort rather than risk commanding a bogus goal if a read fails.
        let mut home = Vec::with_capacity(ids.len());
        if kind.writes() {
            let Some(bus) = self.bus.as_mut() else { return };
            for &id in &ids {
                match bus.read(id, io.pos_addr, io.pos_ty.len()) {
                    Ok(bytes) => home.push(decode_value(&bytes, io.pos_ty) as i32),
                    Err(_) => {
                        self.status =
                            format!("Could not read position of id {id}; aborting R/W benchmark.");
                        self.bench_queue.clear();
                        return;
                    }
                }
            }
        }

        // Sine runs command motion, so enable torque for the duration (it is
        // disabled again in finish_bench). Hold runs do not move and are left
        // untouched. Warn if the model has no torque register to drive.
        if kind.moving() {
            if io.has_torque {
                self.set_torque(&ids, io, true);
            } else {
                self.status = "No Torque Enable register — motors may not move.".into();
            }
        }

        self.bench_run = Some(BenchRun {
            kind,
            duration_secs: self.bench_secs,
            done: 0,
            stats: Stats::default(),
            ids,
            io,
            home,
            amp_deg: self.bench_amp_deg,
            freq_hz: self.bench_freq_hz,
            started: Instant::now(),
        });
        let extra = if kind.moving() && io.has_torque {
            " (torque on)"
        } else {
            ""
        };
        self.status = format!(
            "Running: {} for {:.0}s{}…",
            kind.title(),
            self.bench_secs,
            extra
        );
    }

    pub fn stop_bench(&mut self) {
        if self.bench_run.is_none() {
            return;
        }
        self.bench_queue.clear();
        self.finish_bench();
        self.status = "Benchmark stopped.".into();
    }

    /// Run a single cycle of the active benchmark. Returns true while still running.
    pub fn tick_bench(&mut self) -> bool {
        if self.bench_run.as_ref().is_none_or(|r| r.is_done()) {
            self.finish_bench();
            return false;
        }
        self.run_one_cycle();
        if self.bench_run.as_ref().is_none_or(|r| r.is_done()) {
            self.finish_bench();
            return false;
        }
        true
    }

    fn run_one_cycle(&mut self) {
        // Disjoint borrows of two distinct Option fields.
        let (Some(run), Some(bus)) = (self.bench_run.as_mut(), self.bus.as_mut()) else {
            return;
        };
        let t = run.started.elapsed().as_secs_f64();
        let io = run.io;
        let mut errors = 0usize;
        let started = Instant::now();

        match run.kind {
            BenchKind::PingOne | BenchKind::PingAll => {
                for &id in &run.ids {
                    if !bus.ping(id) {
                        errors += 1;
                    }
                }
            }
            BenchKind::ReadOnePos | BenchKind::ReadAllSeq => {
                for &id in &run.ids {
                    if bus.read(id, io.pos_addr, io.pos_ty.len()).is_err() {
                        errors += 1;
                    }
                }
            }
            BenchKind::ReadAllSync => {
                match bus.sync_read(&run.ids, io.pos_addr, io.pos_ty.len()) {
                    Ok(v) if v.len() == run.ids.len() && v.iter().all(|b| !b.is_empty()) => {}
                    _ => errors += run.ids.len(),
                }
            }
            BenchKind::RwHoldSeq | BenchKind::RwSineSeq => {
                let moving = run.kind.moving();
                for (i, &id) in run.ids.iter().enumerate() {
                    let goal = run.goal(i, t, moving);
                    let bytes = encode_value(goal as i64, io.goal_ty);
                    if bus.write(id, io.goal_addr, &bytes).is_err() {
                        errors += 1;
                    }
                    if bus.read(id, io.pos_addr, io.pos_ty.len()).is_err() {
                        errors += 1;
                    }
                }
            }
            BenchKind::RwHoldSync | BenchKind::RwSineSync => {
                let moving = run.kind.moving();
                let goals: Vec<Vec<u8>> = (0..run.ids.len())
                    .map(|i| encode_value(run.goal(i, t, moving) as i64, io.goal_ty))
                    .collect();
                if bus.sync_write(&run.ids, io.goal_addr, &goals).is_err() {
                    errors += 1;
                }
                match bus.sync_read(&run.ids, io.pos_addr, io.pos_ty.len()) {
                    Ok(v) if v.len() == run.ids.len() && v.iter().all(|b| !b.is_empty()) => {}
                    _ => errors += 1,
                }
            }
        }

        let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        run.done += 1;
        run.stats.errors += errors;
        if errors == 0 {
            run.stats.ok += 1;
            run.stats.samples_ms.push(elapsed_ms);
        }
    }

    fn finish_bench(&mut self) {
        let Some(run) = self.bench_run.take() else {
            return;
        };
        // Restore motors to their captured resting positions after a write sweep.
        if run.kind.writes() && run.io.has_goal {
            if let Some(bus) = self.bus.as_mut() {
                for (&id, &home) in run.ids.iter().zip(run.home.iter()) {
                    let bytes = encode_value(home as i64, run.io.goal_ty);
                    let _ = bus.write(id, run.io.goal_addr, &bytes);
                }
            }
        }

        // Sine runs enabled torque at the start — turn it back off so the bus
        // is left in the state the user found it in.
        if run.kind.moving() {
            let ids = run.ids.clone();
            self.set_torque(&ids, run.io, false);
        }

        let secs = run.elapsed_secs();
        let result = BenchResult {
            kind: run.kind,
            stats: run.stats,
            motors: run.ids.len(),
            iters: run.done,
            secs,
        };
        let errs = result.stats.errors;
        let title = run.kind.title();
        self.bench_results.insert(bench_key(run.kind), result);
        self.status = if errs == 0 {
            format!("Done: {} — {} cycles in {:.1}s, no errors.", title, run.done, secs)
        } else {
            format!(
                "Done: {} — {} cycles in {:.1}s, {} error(s).",
                title, run.done, secs, errs
            )
        };

        // Chain into the next queued benchmark, if any.
        if !self.bench_queue.is_empty() {
            let next = self.bench_queue.remove(0);
            self.start_bench(next);
        }
    }
}
