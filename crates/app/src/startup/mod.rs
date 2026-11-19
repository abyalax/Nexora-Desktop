//! Process start-up concerns that run before any feature exists: command-line flags,
//! single-instance arbitration, leftover-state recovery and the developer demo surface.

pub mod args;
pub mod demo;
pub mod recovery;
pub mod single_instance;
