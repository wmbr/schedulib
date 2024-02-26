use crate::{Time, Job};

use std::cmp::max;
use std::fmt;


/// A job with an assigned starting time and duration
/// Durations should be positive
#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct JobRun {
	pub time: Time,
	pub job: Job,
	pub duration: Time,
}

/// A schedule of jobs on a single machine
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MachineSchedule {
	/// List of job executions, sorted by time.
	/// If jobs can be preempted, the same job may appear in multiple entries.
	pub schedule: Vec<JobRun>,
}

impl MachineSchedule {
	/// Construct a schedule from given processing times.
	/// # Arguments
	/// ptimes: ptimes[i] is the processing time of job i.
	pub fn from_ptimes(ptimes: &[Time]) -> MachineSchedule {
		let mut time = 0;
		let schedule = ptimes.iter().enumerate().map(|(i, d)| {
			time += d;
			JobRun{
				time: time - d,
				job: i,
				duration: *d,
			}
		}).collect();
		MachineSchedule{ schedule }
	}

	/// Construct a schedule from a give job order and processing times.
	/// # Arguments
	/// order: The order of the jobs
	/// ptimes: ptimes[i] is the processing time of job i.
	pub fn from_order_ptimes<I>(order: I, ptimes: &[Time]) -> MachineSchedule 
	where I: Iterator<Item = Job>
	{
		MachineSchedule::from_order_ptimes_releasetimes(
			order,
			ptimes,
			&vec![0; ptimes.len()]
		)
	}

	pub fn new() -> MachineSchedule {
		MachineSchedule { schedule: Vec::new() }
	}

	pub fn from_ptimes_releasetimes(ptimes: &[Time], release_times: &[Time]) -> MachineSchedule {
		MachineSchedule::from_order_ptimes_releasetimes(
			0..ptimes.len(),
			ptimes,
			release_times
		)
	}

	/// Construct a schedule from a given job order and processing times and release times.
	/// # Arguments
	/// order: The order of the jobs
	/// ptimes: ptimes[i] is the processing time of job i.
	/// release_times: release_times[i] is the release time of job i.
	pub fn from_order_ptimes_releasetimes<I>(
		order: I,
		ptimes: &[Time],
		release_times: &[Time]
	) -> MachineSchedule
	where I: Iterator<Item = Job>
	{
		let mut time = 0;
		let schedule = order.map(|job| {
			time = max(time, release_times[job]) + ptimes[job];
			JobRun{
				time: time - ptimes[job],
				job: job,
				duration: ptimes[job],
			}
		}).collect();
		MachineSchedule{ schedule }
	}

	/// Returns the makespan of this MachineSchedule.
	pub fn makespan(&self) -> Time {
		self.schedule.last().map(|run| run.time + run.duration).unwrap_or(0)
	}

	/// Returns the maximum lateness of this MachineSchedule for the given due dates
	///
	/// # Arguments:
	/// * `due_times` A vector containing at position `i` the due date for job `i`.
	pub fn max_lateness(&self, due_times: &[Time]) -> Time {
		self.schedule.iter().map(|run| {
			run.time + run.duration - due_times[run.job]
		}).max().expect("MachineSchedule is empty")
	}

	/// Returns the number of tardy jobs in this MachineSchedule.
	pub fn num_tardy(&self, due_times: &[Time]) -> usize {
		self.schedule.iter().filter(|&run| {
			run.time + run.duration > due_times[run.job]
		}).count()
	}
}

impl fmt::Display for MachineSchedule {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.schedule.is_empty() {
			write!(f, "(Empty MachineSchedule)")
		} else {
			let maxlen = self.makespan().to_string().len();
			for run in self.schedule.iter(){
				writeln!(f,
					"{:len$}-{:len$}: Job #{}",
					run.time,
					run.time + run.duration,
					run.job,
					len = maxlen
				)?;
			}
			Ok(())
		}
	}
}


/// A schedule of jobs on a set of mutliple machines
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MultiMachineSchedule {
	/// The schedule for each machine
	pub machine_schedules: Vec<MachineSchedule>,
}

impl MultiMachineSchedule {
	/// Returns the makespan (i.e. the maximum makespan of any machine).
	pub fn makespan(&self) -> Time {
		self.machine_schedules.iter().map( |s| s.makespan() ).max().unwrap_or(0)
	}

	/// Returns a schedule with no machines
	pub fn new() -> MultiMachineSchedule {
		MultiMachineSchedule { machine_schedules: Vec::new() }
	}

	/// Returns a schedule in which each job is processod on machine 0, 1, 2,... in order
	/// and every machine processes the jobs according to the given `order`.
	///
	/// # Arguments
	/// * order: Order in which jobs are processed by each machine
	/// * ptimes: ptimes[i][j] is the time taken by machine i for job j.
	pub fn from_order_ptimes(order: &[Job], ptimes: &[Vec<Time>]) -> MultiMachineSchedule {
		let m = ptimes.len();
		let mut result = MultiMachineSchedule{
			machine_schedules: Vec::with_capacity(m)
		};
		if m == 0 {
			return result;
		}
		let n = ptimes[0].len();
		let mut ready_times = vec![0; n]; // time when each job is ready to be processed further
		for i in 0..m {
			let mut time = 0;
			let mut schedule = MachineSchedule{ schedule: Vec::with_capacity(n) };
			for &j in order {
				let start = max(time, ready_times[j]);
				schedule.schedule.push( JobRun{
					time: start, 
					job: j,
					duration: ptimes[i][j],
				});
				time = start + ptimes[i][j];
				ready_times[j] = time;
			}
			result.machine_schedules.push(schedule);
		}
		result
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn example_schedule_1() -> MachineSchedule {
		MachineSchedule::from_ptimes_releasetimes(
			&vec![ 5,  6,  7,  3,  6,  2],
			&vec![10, 13, 11, 30,  0, 30]
		)
	}

	#[test]
	fn test_makespan_1() {
		assert_eq!(example_schedule_1().makespan(), 41);
	}

	#[test]
	fn test_lateness_1() {
		let due_times = vec![19, 20, 24, 35, 17, 38];
		assert_eq!(example_schedule_1().max_lateness(&due_times), 22)
	}

	fn example_schedule_2() -> MachineSchedule {
		MachineSchedule::from_ptimes_releasetimes(
			&vec![ 6,  5,  6,  7,  4,  3,  2],
			&vec![ 0, 10, 13, 11, 20, 30, 30]
		)
	}

	#[test]
	fn test_makespan_2() {
		assert_eq!(example_schedule_2().makespan(), 37);
	}

	#[test]
	fn test_lateness_2() {
		let due_times = vec![17, 17, 26, 35, 34, 38, 40];
		assert_eq!(example_schedule_2().max_lateness(&due_times), -2);
	}

	// schedule with preemptions:
	fn example_schedule_3() -> MachineSchedule {
		let schedule = vec![
			JobRun{ time: 0,  job: 0, duration: 5 },
			JobRun{ time: 5,  job: 1, duration: 8 },
			JobRun{ time: 13, job: 0, duration: 9 },
			JobRun{ time: 42, job: 2, duration: 10 },
		];
		MachineSchedule{ schedule }
	}

	#[test]
	fn test_makespan_3() {
		assert_eq!(example_schedule_3().makespan(), 42+10);
	}

	#[test]
	fn test_lateness_3() {
		let due_times = vec![20, 15, 52];
		assert_eq!(example_schedule_3().max_lateness(&due_times), 13+9-20);
	}

	// schedule with preemptions:
	fn example_schedule_4() -> MachineSchedule {
		let schedule = vec![
			JobRun{ time: 3,  job: 0, duration: 13 },
			JobRun{ time: 16, job: 1, duration: 8 },
			JobRun{ time: 24, job: 0, duration: 7 },
		];
		MachineSchedule{ schedule }
	}

	#[test]
	fn test_schedule_makespan_2() {
		assert_eq!(example_schedule_4().makespan(), 24 + 7);
	}

	#[test]
	fn test_schedule_lateness_2() {
		let due_times = vec![25, 24];
		assert_eq!(example_schedule_4().max_lateness(&due_times), 24 + 7 - 25);
	}

	#[test]
	fn test_multischedule_from_order_ptimes() {
		let ptimes = vec![
			vec![9, 1, 9, 4],
			vec![6, 3, 5, 5],
		];
		let order = vec![2, 1, 3, 0];
		let result = MultiMachineSchedule::from_order_ptimes(&order, &ptimes);
		assert_eq!(result.machine_schedules[0], MachineSchedule::from_order_ptimes(order.into_iter(), &ptimes[0]));
		assert_eq!(result.machine_schedules[1].schedule[3].time, 23);

	}
}
