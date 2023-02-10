use std::cmp::max;

type Time = isize; // allowing negative times can be useful occasionally

/// Job with release time, processing time and due time
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Job {
	pub release_time: Time,
	pub processing_time: Time, // should be nonnegative
	pub due_time: Time,
}


impl Job {
	pub fn new(release_time: Time, processing_time: Time, due_time: Time) -> Job {
		Job {
			release_time,
			processing_time,
			due_time,
		}
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JobList {
	pub jobs: Vec<Job>,
}

impl JobList {
	/// Returns the makespan of this JobList (if all jobs are executed on a single machine).
	pub fn makespan(&self) -> Time {
		let mut t = 0; // current time

		for job in self.jobs.iter() {
			if job.release_time > t {
				t = job.release_time + job.processing_time;
			} else {
				t += job.processing_time;
			}
		}
		t
	}

	pub fn lateness(&self) -> Time {
		let mut t = 0; // current time
		let mut lateness = Time::MIN;
		for job in self.jobs.iter() {
			if job.release_time > t {
				t = job.release_time + job.processing_time;
			} else {
				t += job.processing_time;
			}
			lateness = max(lateness, t - job.due_time);
		}
		lateness
	}
}

/// A job execution schedule for a single machine with possible preemptions, assigning to every job one or multiple execution times.
/// If a job is assigned multiple execution times, then it was preempted by some other job in between.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JobSchedule {
	pub jobs: Vec<Job>,
	/// For every time a job is started or resumed, this contains an entry with the time and the job's position in [job_list].
	pub timetable: Vec<(Time, usize)>,
}

impl JobSchedule {
	/// compute the maximum lateness of the schedule
	pub fn lateness(&self) -> Time {
		let mut lateness = Time::MIN;
		let mut processing_times_remaining : Vec<Time> =
			self.jobs.iter().map(|job| job.processing_time).collect();
		for pair in self.timetable.windows(2) {
			let [(time1, index1), (time2, _)] = <&[_; 2]>::try_from(pair).unwrap();
			lateness = max(
				lateness,
				time1 + processing_times_remaining[*index1] - self.jobs[*index1].due_time
			);
			processing_times_remaining[*index1] -= time2 - time1;
			if processing_times_remaining[*index1] < 0 {
				processing_times_remaining[*index1] = 0;
			}
		}
		let (last_time, last_index) = match self.timetable.last() {
			Some(x) => x,
			None => return 0,
		};
		lateness = max(
			lateness,
			last_time + processing_times_remaining[*last_index] - self.jobs[*last_index].due_time
		);
		lateness
	}

	/// compute the makespan of the schedule, i.e., the time at which all jobs are completed
	pub fn makespan(&self) -> Time {
		let (last_time, last_index) = match self.timetable.last() {
			Some(x) => x,
			None => return 0,
		};
		let mut processing_time_remaining = self.jobs[*last_index].processing_time;
		for pair in self.timetable.windows(2) {
			let [(time1, index1), (time2, _)] = <&[_; 2]>::try_from(pair).unwrap();
			if *index1 == *last_index {
				processing_time_remaining -= time2 - time1;
				if processing_time_remaining < 0 {
					processing_time_remaining = 0;
				}
			}
		}
		*last_time + processing_time_remaining
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	fn joblist1() -> JobList {
		JobList {
			jobs: vec![
				Job::new(10, 5, 19),
				Job::new(13, 6, 20),
				Job::new(11, 7, 24),
				Job::new(30, 3, 35),
				Job::new(0, 6, 17),
				Job::new(30, 2, 38),
			],
		}
	}

	#[test]
	fn test_makespan_1() {
		assert_eq!(joblist1().makespan(), 41);
	}

	#[test]
	fn test_lateness_1() {
		assert_eq!(joblist1().lateness(), 22)
	}

	fn joblist2() -> JobList {
		JobList {
			jobs: vec![
				Job::new(0, 6, 17),
				Job::new(10, 5, 17),
				Job::new(13, 6, 26),
				Job::new(11, 7, 35),
				Job::new(20, 4, 34),
				Job::new(30, 3, 38),
				Job::new(30, 2, 40),
			],
		}
	}

	#[test]
	fn test_makespan_2() {
		assert_eq!(joblist2().makespan(), 37);
	}

	#[test]
	fn test_lateness_2() {
		assert_eq!(joblist2().lateness(), -2);
	}

	fn schedule1() -> JobSchedule {
		let jobs = vec![
			Job::new(0, 14, 20), // 0
			Job::new(5, 8, 15),  // 1
			Job::new(42, 10, 52),  // 2
		];
		let timetable = vec![
			(0, 0),
			(5, 1),
			(13, 0),
			(42, 2),
		];
		JobSchedule{
			jobs,
			timetable,
		}
	}

	#[test]
	fn test_schedule_makespan_1() {
		assert_eq!(schedule1().makespan(), 42+10);
	}

	#[test]
	fn test_schedule_lateness_1() {
		assert_eq!(schedule1().lateness(), 22-20);
	}

	fn schedule2() -> JobSchedule {
		let jobs = vec![
			Job::new(3, 20, 25), // 0
			Job::new(5, 8, 24),  // 1
		];
		let timetable = vec![
			(3, 0),
			(16, 1),
			(24, 0),
		];
		JobSchedule{
			jobs,
			timetable,
		}
	}

	#[test]
	fn test_schedule_makespan_2() {
		assert_eq!(schedule2().makespan(), 24 + 7);
	}

	#[test]
	fn test_schedule_lateness_2() {
		assert_eq!(schedule2().lateness(), 6);
	}
}
