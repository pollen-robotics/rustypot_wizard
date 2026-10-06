//! Communication benchmarks for a servo bus.
//!
//! Each benchmark runs a number of *cycles*. A cycle is one logical operation
//! (a ping, a position read, a full read/write loop over all motors, …). Cycle
//! wall-clock time is sampled and aggregated into latency statistics plus an
//! achievable rate (Hz). Failed sub-transactions are counted separately so the
//! latency numbers stay clean (only fully successful cycles are sampled).

use std::f64::consts::TAU;
use std::time::Instant;

use crate::registers::RegType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchKind {
    /// Ping the selected motor.
    PingOne,
    /// Ping every discovered motor (one cycle = full sweep).
    PingAll,
    /// Read present position of the selected motor.
    ReadOnePos,
    /// Read present position of every motor, one read per motor.
    ReadAllSeq,
    /// Read present position of every motor in a single sync_read.
    ReadAllSync,
    /// Per motor: write the resting position (no motion) then read it back.
    RwHoldSeq,
    /// One sync_write of resting positions (no motion) + one sync_read.
    RwHoldSync,
    /// Per motor: write a sine goal then read present position (sequential round trips).
    RwSineSeq,
    /// One sync_write of sine goals + one sync_read of positions (fast control loop).
    RwSineSync,
}

impl BenchKind {
    pub const ALL: &'static [BenchKind] = &[
        BenchKind::PingOne,
        BenchKind::PingAll,
        BenchKind::ReadOnePos,
        BenchKind::ReadAllSeq,
        BenchKind::ReadAllSync,
        BenchKind::RwHoldSeq,
        BenchKind::RwHoldSync,
        BenchKind::RwSineSeq,
        BenchKind::RwSineSync,
    ];

    pub fn title(self) -> &'static str {
        match self {
            BenchKind::PingOne => "Ping one motor",
            BenchKind::PingAll => "Ping all motors (sweep)",
            BenchKind::ReadOnePos => "Read one position",
            BenchKind::ReadAllSeq => "Read all positions (sequential)",
            BenchKind::ReadAllSync => "Read all positions (sync)",
            BenchKind::RwHoldSeq => "R/W loop, hold (sequential)",
            BenchKind::RwHoldSync => "R/W loop, hold (sync)",
            BenchKind::RwSineSeq => "R/W loop, sine (sequential)",
            BenchKind::RwSineSync => "R/W loop, sine (sync)",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            BenchKind::PingOne => "Round-trip latency of a single ping to the selected motor.",
            BenchKind::PingAll => "Latency to ping every motor on the bus once (full sweep).",
            BenchKind::ReadOnePos => "Round-trip latency of one Present Position read.",
            BenchKind::ReadAllSeq => {
                "Read Present Position from each motor with individual reads. Cost scales with motor count."
            }
            BenchKind::ReadAllSync => {
                "Read Present Position from all motors in one sync_read packet — compare against sequential."
            }
            BenchKind::RwHoldSeq => {
                "Per motor: write the resting Goal Position (no motion) then read Present Position. Safe full R/W loop — try this first."
            }
            BenchKind::RwHoldSync => {
                "One sync_write of resting positions (no motion) + one sync_read. Safe fast control loop — try this first."
            }
            BenchKind::RwSineSeq => {
                "Per motor: write a small sine Goal Position then read Present Position. Torque is enabled for the run and disabled afterwards."
            }
            BenchKind::RwSineSync => {
                "One sync_write of sine goals + one sync_read of positions. The fast real-time control loop. Torque is enabled for the run and disabled afterwards."
            }
        }
    }

    /// Does this benchmark involve every motor on the bus (vs. just the selected one)?
    pub fn uses_all(self) -> bool {
        !matches!(self, BenchKind::PingOne | BenchKind::ReadOnePos)
    }

    /// Does this benchmark write Goal Position (and therefore need home capture)?
    pub fn writes(self) -> bool {
        matches!(
            self,
            BenchKind::RwHoldSeq
                | BenchKind::RwHoldSync
                | BenchKind::RwSineSeq
                | BenchKind::RwSineSync
        )
    }

    /// Does this benchmark drive a sine trajectory (vs. holding the resting position)?
    pub fn moving(self) -> bool {
        matches!(self, BenchKind::RwSineSeq | BenchKind::RwSineSync)
    }
}

/// Resolved register addresses for the bus under test, derived once from the
/// selected motor's model. Benchmarks assume a homogeneous bus (the common case).
#[derive(Debug, Clone, Copy)]
pub struct BenchIo {
    pub pos_addr: u8,
    pub pos_ty: RegType,
    pub has_pos: bool,
    pub goal_addr: u8,
    pub goal_ty: RegType,
    pub has_goal: bool,
    pub torque_addr: u8,
    pub torque_ty: RegType,
    pub has_torque: bool,
    pub deg_per_count: f64,
}

/// Aggregated latency statistics over sampled cycles.
#[derive(Debug, Default, Clone)]
pub struct Stats {
    /// Per-cycle wall-clock in milliseconds (successful cycles only).
    pub samples_ms: Vec<f64>,
    /// Cycles that completed with no sub-transaction error.
    pub ok: usize,
    /// Number of failed sub-transactions (reads/writes/pings/syncs).
    pub errors: usize,
}

impl Stats {
    pub fn min(&self) -> Option<f64> {
        self.samples_ms.iter().copied().fold(None, |acc, x| {
            Some(acc.map_or(x, |a: f64| a.min(x)))
        })
    }

    pub fn max(&self) -> Option<f64> {
        self.samples_ms.iter().copied().fold(None, |acc, x| {
            Some(acc.map_or(x, |a: f64| a.max(x)))
        })
    }

    pub fn mean(&self) -> Option<f64> {
        if self.samples_ms.is_empty() {
            return None;
        }
        Some(self.samples_ms.iter().sum::<f64>() / self.samples_ms.len() as f64)
    }

    pub fn std(&self) -> Option<f64> {
        let m = self.mean()?;
        if self.samples_ms.len() < 2 {
            return Some(0.0);
        }
        let var = self
            .samples_ms
            .iter()
            .map(|x| (x - m) * (x - m))
            .sum::<f64>()
            / self.samples_ms.len() as f64;
        Some(var.sqrt())
    }

    /// Percentile in [0,1] using nearest-rank on a sorted copy.
    pub fn percentile(&self, p: f64) -> Option<f64> {
        if self.samples_ms.is_empty() {
            return None;
        }
        let mut v = self.samples_ms.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let rank = (p * (v.len() as f64 - 1.0)).round() as usize;
        Some(v[rank.min(v.len() - 1)])
    }

    /// Achievable cycle rate in Hz, from the mean cycle time.
    pub fn hz(&self) -> Option<f64> {
        self.mean().filter(|m| *m > 0.0).map(|m| 1000.0 / m)
    }
}

/// A finished benchmark result, kept for display.
#[derive(Debug, Clone)]
pub struct BenchResult {
    #[allow(dead_code)] // kept for completeness; display keys off the selected kind
    pub kind: BenchKind,
    pub stats: Stats,
    pub motors: usize,
    /// Cycles completed during the run.
    pub iters: usize,
    /// Wall-clock the run actually took, in seconds.
    pub secs: f64,
}

/// Live state of a benchmark being pumped one cycle per call.
pub struct BenchRun {
    pub kind: BenchKind,
    /// How long to keep running, in seconds.
    pub duration_secs: f64,
    pub done: usize,
    pub stats: Stats,
    pub ids: Vec<u8>,
    pub io: BenchIo,
    /// Resting position per id, captured at start (one entry per id, write benchmarks only).
    pub home: Vec<i32>,
    pub amp_deg: f64,
    pub freq_hz: f64,
    pub started: Instant,
}

impl BenchRun {
    pub fn elapsed_secs(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    pub fn is_done(&self) -> bool {
        self.elapsed_secs() >= self.duration_secs
    }

    pub fn ratio(&self) -> f64 {
        if self.duration_secs <= 0.0 {
            return 1.0;
        }
        (self.elapsed_secs() / self.duration_secs).clamp(0.0, 1.0)
    }

    /// Goal (raw counts) for motor index `i` at elapsed time `t`.
    /// For a "hold" run this is exactly the resting position; for a sine run a
    /// small phase-shifted offset is added so the bus moves as a travelling wave.
    pub fn goal(&self, i: usize, t: f64, moving: bool) -> i32 {
        let home = self.home[i];
        if !moving {
            return home;
        }
        let amp_counts = if self.io.deg_per_count > 0.0 {
            self.amp_deg / self.io.deg_per_count
        } else {
            0.0
        };
        let n = self.ids.len().max(1) as f64;
        let phase = TAU * i as f64 / n;
        let off = amp_counts * (TAU * self.freq_hz * t + phase).sin();
        home + off.round() as i32
    }
}
