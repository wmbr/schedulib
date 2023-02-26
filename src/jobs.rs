use std::cmp::max;
use std::fmt;

pub type Time = isize; // allowing negative times can be useful occasionally
pub type Job = usize; // jobs are ids
pub type Machine = usize; // machines are ids

/// A job with an assigned starting time and duration
/// Durations should be positive
#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct JobRun {
	pub time: Time,
	pub job: Job,
	pub duration: Time,
}

/// A schedule of jobs on a single machine (without preemptions)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JobSchedule {
	/// List of job executions, sorted by time.
	/// If jobs can be preempted, the same job may appear in multiple entries.
	pub schedule: Vec<JobRun>,
}

impl JobSchedule {
	pub fn from_durations(durations: &[Time]) -> JobSchedule {
		let mut time = 0;
		let schedule = durations.iter().enumerate().map(|(i, d)| {
			time += d;
			JobRun{
				time: time - d,
				job: i,
				duration: *d,
			}
		}).collect();
		JobSchedule{ schedule }
	}

	pub fn new() -> JobSchedule {
		JobSchedule { schedule: Vec::new() }
	}

	pub fn from_durations_releasetimes(durations: &[Time], release_times: &[Time]) -> JobSchedule {
		JobSchedule::from_order_durations_releasetimes(
			&(0..durations.len()).collect::<Vec<Job>>(),
			durations,
			release_times
		)
	}

	pub fn from_order_durations_releasetimes(
		order: &[Job],
		durations: &[Time],
		release_times: &[Time]
	) -> JobSchedule
	{
		let mut time = 0;
		let schedule = order.iter().map(|job| {
			time = max(time, release_times[*job]) + durations[*job];
			JobRun{
				time: time - durations[*job],
				job: *job,
				duration: durations[*job],
			}
		}).collect();
		JobSchedule{ schedule }
	}

	/// Returns the makespan of this JobSchedule.
	pub fn makespan(&self) -> Time {
		self.schedule.last().map(|run| run.time + run.duration).unwrap_or(0)
	}

	/// Returns the maximum lateness of this JobSchedule for the given due dates
	///
	/// # Arguments:
	/// * `due_times` A vector containing at position `i` the due date for job `i`.
	pub fn lateness(&self, due_times: &[Time]) -> Time {
		self.schedule.iter().map(|run| {
			run.time + run.duration - due_times[run.job]
		}).max().expect("JobSchedule is empty")
	}
}

impl fmt::Display for JobSchedule {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.schedule.is_empty() {
			write!(f, "(Empty JobSchedule)")
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

#[cfg(test)]
mod tests {

	use super::*;

	fn example_schedule_1() -> JobSchedule {
		JobSchedule::from_durations_releasetimes(
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
		assert_eq!(example_schedule_1().lateness(&due_times), 22)
	}

	fn example_schedule_2() -> JobSchedule {
		JobSchedule::from_durations_releasetimes(
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
		assert_eq!(example_schedule_2().lateness(&due_times), -2);
	}

	// schedule with preemptions:
	fn example_schedule_3() -> JobSchedule {
		let schedule = vec![
			JobRun{ time: 0,  job: 0, duration: 5 },
			JobRun{ time: 5,  job: 1, duration: 8 },
			JobRun{ time: 13, job: 0, duration: 9 },
			JobRun{ time: 42, job: 2, duration: 10 },
		];
		JobSchedule{ schedule }
	}

	#[test]
	fn test_makespan_3() {
		assert_eq!(example_schedule_3().makespan(), 42+10);
	}

	#[test]
	fn test_lateness_3() {
		let due_times = vec![20, 15, 52];
		assert_eq!(example_schedule_3().lateness(&due_times), 13+9-20);
	}

	// schedule with preemptions:
	fn example_schedule_4() -> JobSchedule {
		let schedule = vec![
			JobRun{ time: 3,  job: 0, duration: 13 },
			JobRun{ time: 16, job: 1, duration: 8 },
			JobRun{ time: 24, job: 0, duration: 7 },
		];
		JobSchedule{ schedule }
	}

	#[test]
	fn test_schedule_makespan_2() {
		assert_eq!(example_schedule_4().makespan(), 24 + 7);
	}

	#[test]
	fn test_schedule_lateness_2() {
		let due_times = vec![25, 24];
		assert_eq!(example_schedule_4().lateness(&due_times), 24 + 7 - 25);
	}
}
