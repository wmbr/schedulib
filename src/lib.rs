pub type Time = isize; // allowing negative times can be useful occasionally
pub type Job = usize; // jobs are ids
pub type Machine = usize; // machines are ids

pub mod schedule;
pub use schedule::*;
pub mod single_machine;
pub mod unrelated_machines;
pub mod flow_shop;