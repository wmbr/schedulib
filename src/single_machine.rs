use crate::jobs::{Time, Job, JobSchedule, JobScheduleWithPreemptions};
use std::cmp;
use std::collections::BinaryHeap;

#[derive(Eq)]
struct SchrageJob {
	pub job: Job,
}

impl Ord for SchrageJob {
	// Order by "earliest due date first",
	// using "longest processing time first" as tiebreaker
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		if self.job.due_time == other.job.due_time {
			self.job.processing_time.cmp(&other.job.processing_time)
		} else {
			self.job.due_time.cmp(&other.job.due_time)
		}
	}
}

impl PartialOrd for SchrageJob {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for SchrageJob {
	fn eq(&self, other: &Self) -> bool {
		self.cmp(other) == cmp::Ordering::Equal
	}
}



/// Schrage's heuristic for 1|r_j|L_max.
/// Schedules jobs on a single machine in an attempt to minimze the maximum lateness.
/// Runs in O(n log n) time for n jobs.
/// If all release times are identical, this is guaranteed to produce the optimum solution.
///
/// # Arguments
///
/// * `jobs`: A list of jobs.
///
pub fn schrage(mut jobs: Vec<Job>) -> JobSchedule {
	// sort by descending release time
	// because we want to pop the jobs with lowest release time first
	jobs.sort_unstable_by_key(|x| cmp::Reverse(x.release_time));
	// A list of jobs that in a current moment are ready to run, sorted by "highest priority first"
	let mut ready_to_run = BinaryHeap::new();
	// Time tracking variable
	let mut t: Time = 0;
	// The final sequence in which the jobs should be run
	let mut schedule = Vec::new();

	// Iterate over jobs in order of release time
	while !jobs.is_empty() || !ready_to_run.is_empty() {
		// Find all jobs that are available
		while !jobs.is_empty()
			&& jobs.last().unwrap().release_time <= t
		{
			ready_to_run.push(
				cmp::Reverse(SchrageJob{ job: jobs.pop().unwrap() })
			);
		}
		// If there are jobs that are ready to run, schedule them
		match ready_to_run.pop() {
			Some(cmp::Reverse(sjob)) => {
				schedule.push(sjob.job);
				t += sjob.job.processing_time;
			},
			None => {
				// If there aren't any jobs that can be run,
				// skip to when the nearest job is available.
				// Note that ready_to_run cannot be empty at this point.
				t = jobs.last().unwrap().release_time;
			}
		};
	}
	JobSchedule::new(schedule)
}


/// EDD scheduler with preemptions.
/// Produces an optimum schedule for 1|pmtn,r_j|L_max.
/// Sorts jobs by "earlist due date first" (in case of ties, longer computation times are scheduled first).
/// Runs in O(n log n) time for n jobs.
///
/// # Arguments
///
/// * `jobs`: A list of jobs.
///
pub fn edd_preemptive(mut jobs: Vec<Job>) -> JobScheduleWithPreemptions {
	// sort by ascending release time
	jobs.sort_unstable_by_key(|x| x.release_time);
	// A list of jobs that in a current moment are ready to run, sorted by descending priority
	// Together with each job we store its index (in `jobs`).
	let mut ready_to_run = BinaryHeap::new();
	// Time tracking variable
	let mut t: Time = 0;
	// The final timetable
	let mut timetable: Vec<(Time, usize)> = Vec::new();
	// index of the next job to become available
	let mut job_index = 0;
	// Iterate over all of the jobs until we ran out of them
	while job_index < jobs.len() || !ready_to_run.is_empty() {
		// Find all jobs that are available
		while job_index < jobs.len()
			&& jobs[job_index].release_time <= t
		{
			ready_to_run.push((
				cmp::Reverse(SchrageJob{ job: jobs[job_index] }),
				job_index,
			));
			job_index += 1;
		}
		// If there are jobs that are ready to run schedule them
		match ready_to_run.pop() {
			Some((cmp::Reverse(mut sjob), i)) => {
				// Schedule that job unless it is already scheduled
				if timetable.is_empty() || timetable.last().unwrap().1 != i {
					timetable.push((t, i));
				}
				t += sjob.job.processing_time;
				// check if a new job arrives before this one is done
				if job_index < jobs.len() {
					let next_delivery = jobs[job_index].release_time;
					if next_delivery < t {
						// add this job back to the heap with the remaining processing time
						sjob.job.processing_time = t - next_delivery;
						ready_to_run.push((cmp::Reverse(sjob), i));
						t = next_delivery;
					}
				}
			},
			None => {
				// If there aren't any jobs that can be run,
				// skip to when the nearest job is available
				// note that job_index < jobs.len() is guaranteed here
				t = jobs[job_index].release_time;
			}
		};
	}
	JobScheduleWithPreemptions{
		jobs,
		timetable,
	}
}


/// Carlier's algorithm for 1|r_j|L_max
/// Uses Schrage's heuristic and a branch-and-bound approach to solve the problem.
/// Note that the worst-case running time is exponential (the problem is strongly NP-hard).
/// 
/// See [J. Carlier: "The one-machine sequencing problem" (1982); doi:10.1016/S0377-2217(82)80007-6]
///
/// # Arguments
///
/// * `jobs`: A list of jobs.
///
pub fn carlier(jobs: Vec<Job>) -> JobSchedule {
	if jobs.is_empty() {
		return JobSchedule::new(vec![])
	}
	let schedule = schrage(jobs);
	let jobs = &schedule.jobs;
	let (a, p) = critical_path(&schedule);

	// find last job on the critical path with a later due date than p
	let c = match (a..p).rev().find(|i| {
		jobs[*i].due_time > jobs[p].due_time
	}) {
		None => return schedule, // found schedule is already optimal
		Some(c) => c,
	};

	unimplemented!()
}


fn critical_path(schedule: &JobSchedule) -> (usize, usize) {
	let jobs = &schedule.jobs;
	let latenesses = jobs.iter().enumerate().map(|(i, job)|
		(i, schedule.starting_times[i] + job.processing_time - job.due_time)
	);
	// p is the index of the job of maximum lateness
	let (p, _) = latenesses.max_by_key(
		|(i, lateness)| *lateness
	).expect("job list is empty");

	// find last job a <= p which had idle time before it
	let a = (1..=p).rev().find(|i| {
		jobs[*i].release_time > schedule.starting_times[*i-1] + jobs[*i-1].processing_time
	}).unwrap_or(0);
	(a, p)
}


#[cfg(test)]
mod tests {
	use super::*;

	fn jobs1() -> Vec<Job> {
		vec![
			Job::new(10, 5, 15),
			Job::new(13, 6, 25),
			Job::new(11, 7, 32),
			Job::new(20, 4, 24),
			Job::new(30, 3, 36),
			Job::new(0, 6, 17), 
			Job::new(31, 1, 33), 
		]
	}

	#[test]
	fn test_schrage_1() {
		let expected_result = JobSchedule::new(vec![
			Job::new(0, 6, 17),
			Job::new(10, 5, 15),
			Job::new(13, 6, 25),
			Job::new(20, 4, 24),
			Job::new(11, 7, 32),
			Job::new(31, 1, 33),
			Job::new(30, 3, 36),
		]);
		let result = schrage(jobs1());
		assert_eq!(result, expected_result);
	}

	#[test]
	fn test_edd_preemptive_1() {
		let jobs = vec![
			Job::new(0, 6, 17),  //0
			Job::new(10, 5, 15), //1
			Job::new(11, 7, 32), //2
			Job::new(13, 6, 25), //3
			Job::new(20, 4, 24), //4
			Job::new(30, 3, 36), //5
			Job::new(31, 1, 33), //6
		];
		let timetable = vec![
			(0, 0),
			(10, 1),
			(15, 3),
			(20, 4),
			(24, 3),
			(25, 2),
			(32, 6),
			(33, 5),
		];
		let expected_result = JobScheduleWithPreemptions{ jobs, timetable };
		let result = edd_preemptive(jobs1());
		assert_eq!(result, expected_result);
	}

	#[test]
	fn test_critical_path() {
		let schedule = JobSchedule::new(jobs1());
		assert_eq!(critical_path(&schedule), (0, 5));
	}
}
