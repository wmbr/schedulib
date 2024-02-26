use crate::jobs::{Time, Job, MachineSchedule};

use std::collections::BinaryHeap;

/// Hodgson's algorithm for solving 1||num_tardy optimally in O(n log n) time.
///
/// See Blazewicz et al, "Handbook on Scheduling", alg. 4.3.6.
pub fn schedule_hodgson(
	processing_times: &[Time],
	due_times: &[Time]
) -> MachineSchedule
{
	let n = processing_times.len();
	// vector of jobs
	let mut jobs : Vec<Job> = (0..n).collect();
	// sort by earliest due time last, because we will iterate back-to-front
	jobs.sort_unstable_by_key(|&job| -due_times[job]);

	// the jobs that finish on time in our final schedule, ordered by processing time
	let mut jobs_on_time = BinaryHeap::new();
	let mut num_late = 0;

	let mut duration = 0;
	for i in (0..n).rev() {
		let job = jobs[i];
		jobs_on_time.push((processing_times[job], job));
		duration += processing_times[job];
		if duration > due_times[job] {
			// if not all jobs can be on time, have the longest job be late
			let (pt, longest_job) = jobs_on_time.pop().unwrap();
			duration -= pt;

			num_late += 1;
			// we store the late jobs at the end of the jobs vector
			// (these elements were already traversed)
			jobs[n - num_late] = longest_job;
		}
	}
	// copy jobs on time to the front of the result vector
	for (i, &(_, job)) in jobs_on_time.into_vec().iter().enumerate() {
		jobs[i] = job;
	}
	// restore due time order for the jobs on time
	jobs[0..n-num_late].sort_unstable_by_key(|&job| due_times[job]);
	MachineSchedule::from_order_durations(
		jobs.into_iter(),
		&processing_times
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn example_1() -> (Vec<Time>, Vec<Time>) {
		// this is example 4.3.7 in Blazewicz et al, "Handbook on Scheduling"
		(
			//    0   1   2   3   4   5   6   7
			vec![10,  6,  3,  1,  4,  8,  7,  6], // processing
			vec![35, 20, 11,  8,  6, 26, 28,  9], // due
		)
	}

	#[test]
	fn test_hodgson_example_1() {
		let (p, d) = example_1();
		let expected_order = vec![4, 3, 2, 1, 6, 0]; // the remaining two can be in arbitrary order
		let result = schedule_hodgson(&p, &d);
		let order : Vec<Job> = result.schedule.iter().map(|&jr| jr.job).collect();
		assert_eq!(order[..6], expected_order);
	}
}