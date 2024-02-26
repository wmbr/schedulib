use crate::jobs::{Time, Job, MachineSchedule, JobRun};
use std::collections::BinaryHeap;


/// EDD scheduler with preemptions.
/// Produces an optimum schedule for 1|pmtn,r_j|L_max.
/// Sorts jobs by "earlist due date first" (in case of ties, longer computation times are scheduled first).
/// Runs in O(n log n) time for n jobs.
///
/// # Arguments
///
/// * `processing_times`: The processing times of the jobs
/// * `release_times`: The release times of the jobs
/// * `due_times`: due times of the jobs
///
pub fn edd_preemptive(
	mut processing_times: Vec<Time>,
	release_times: &[Time],
	due_times: &[Time]
) -> MachineSchedule
{
	let mut jobs: Vec<Job> = (0..processing_times.len()).collect();
	// sort by descending release time
	// because we want to pop the jobs with lowest release time first
	jobs.sort_unstable_by_key(|&job| -release_times[job]);
	// A list of jobs that in a current moment are ready to run,
	// sorted by "earliest due time first",
	let mut ready_to_run = BinaryHeap::new();
	// Time tracking variable
	let mut t: Time = 0;
	// The final schedule
	let mut schedule: Vec<JobRun> = Vec::new();
	// Iterate over jobs in order of release time
	while !jobs.is_empty() || !ready_to_run.is_empty() {
		// Find all jobs that are available
		while !jobs.is_empty()
			&& release_times[*jobs.last().unwrap()] <= t
		{
			let job = jobs.pop().unwrap();
			// the first tuple entry is just to determine the order
			ready_to_run.push((	-due_times[job], job ));
		}
		// If there are jobs that are ready to run schedule them
		match ready_to_run.pop() {
			Some((_, job)) => {
				// If that job is alread scheduled, just extend its duration
				if !schedule.is_empty() && schedule.last().unwrap().job == job {
					schedule.last_mut().unwrap().duration += processing_times[job];
				} else {
					schedule.push(JobRun {
						time: t,
						job,
						duration: processing_times[job]
					});
				}
				t += processing_times[job];
				// check if a new job arrives before this one is done
				if !jobs.is_empty() {
					let next_delivery = release_times[*jobs.last().unwrap()];
					if next_delivery < t {
						// add this job back to the heap with the remaining processing time:
						processing_times[job] = t - next_delivery;
						ready_to_run.push(( -due_times[job], job ));
						// shorten duration of the scheduled run accordingly:
						schedule.last_mut().unwrap().duration -= processing_times[job];
						t = next_delivery;
					}
				}
			},
			None => {
				// If there aren't any jobs that can be run,
				// skip to when the nearest job is available
				// Note that ready_to_run cannot be empty at this point.
				t = release_times[*jobs.last().unwrap()];
			}
		};
	}
	MachineSchedule{ schedule }
}

#[cfg(test)]
mod tests {
	use super::*;

	fn example_1() -> (Vec<Time>, Vec<Time>, Vec<Time>) {
		(
			//    0   1   2   3   4   5   6
			vec![ 5,  6,  7,  4,  3,  6,  1], // processing
			vec![10, 13, 11, 20, 30,  0, 31], // release
			vec![15, 25, 32, 24, 36, 17, 33], // due
		)
	}
	
	#[test]
	fn test_edd_preemptive_1() {
		let (p, r, d) = example_1();
		let expected_result = MachineSchedule{
			schedule: vec![
				JobRun{ time:  0, job: 5, duration: 6 },
				JobRun{ time: 10, job: 0, duration: 5 },
				JobRun{ time: 15, job: 1, duration: 5 },
				JobRun{ time: 20, job: 3, duration: 4 },
				JobRun{ time: 24, job: 1, duration: 1 },
				JobRun{ time: 25, job: 2, duration: 7 },
				JobRun{ time: 32, job: 6, duration: 1 },
				JobRun{ time: 33, job: 4, duration: 3 },
			]
		};
		let result = edd_preemptive(p, &r, &d);
		assert_eq!(result, expected_result);
	}
}