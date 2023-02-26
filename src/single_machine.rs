use crate::jobs::{Time, Job, JobSchedule, JobRun};
use std::cmp::{max, min, Reverse};
use std::collections::BinaryHeap;


/// Schrage's heuristic for 1|r_j|L_max.
/// Schedules jobs on a single machine in an attempt to minimze the maximum lateness.
/// Runs in O(n log n) time for n jobs.
/// If all release times are identical, this is guaranteed to produce the optimum solution.
///
/// # Arguments
///
/// * `jobs`: A list of jobs.
///
pub fn schrage(
	processing_times: &[Time],
	release_times: &[Time],
	due_times: &[Time]
) -> JobSchedule
{
	let mut jobs: Vec<Job> = (0..processing_times.len()).collect();
	// sort by descending release time
	// because we want to pop the jobs with lowest release time first
	jobs.sort_unstable_by_key(|&job| -release_times[job]);
	// A list of jobs that in a current moment are ready to run,
	// sorted by "earliest due time first",
	// using "longest processing time first" as tiebreaker.
	let mut ready_to_run = BinaryHeap::new();
	// Time tracking variable
	let mut t: Time = 0;
	// The final sequence in which the jobs should be run
	let mut schedule = Vec::new();

	// Iterate over jobs in order of release time
	while !jobs.is_empty() || !ready_to_run.is_empty() {
		// Find all jobs that are available
		while !jobs.is_empty()
			&& release_times[*jobs.last().unwrap()] <= t
		{
			let job = jobs.pop().unwrap();
			// first and second tuple entry are just to determine the correct order
			ready_to_run.push(
				( -due_times[job], processing_times[job], job )
			);
		}
		// If there are jobs that are ready to run, schedule them
		match ready_to_run.pop() {
			Some((_, _, job)) => {
				schedule.push(job);
				t += processing_times[job];
			},
			None => {
				// If there aren't any jobs that can be run,
				// skip to when the nearest job is available.
				// Note that ready_to_run cannot be empty at this point.
				t = release_times[*jobs.last().unwrap()];
			}
		};
	}
	JobSchedule::from_order_durations_releasetimes(&schedule, processing_times, release_times)
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
pub fn edd_preemptive(
	mut processing_times: Vec<Time>,
	release_times: &[Time],
	due_times: &[Time]
) -> JobSchedule
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
	JobSchedule{ schedule }
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
pub fn carlier(processing_times: &[Time], release_times: &[Time], due_times: &[Time]) -> JobSchedule {
	if processing_times.is_empty() {
		return JobSchedule{ schedule: vec![] }
	}
	let mut subproblems = BinaryHeap::new();
	subproblems.push( Reverse((
		Time::MIN,
		CarlierNode{
			release_times: release_times.to_vec(),
			due_times: due_times.to_vec(),
		}
	)));
	let mut best_lateness = Time::MAX;
	let mut best_schedule = None;
	while let Some(Reverse((lower_bound, node))) = subproblems.pop() {
		if lower_bound >= best_lateness {
			continue;
		}
		let result = carlier_iteration(
			processing_times,
			node.release_times,
			node.due_times,
			best_lateness
		);
		let lateness = result.schedule.lateness(due_times);
		if lateness < best_lateness {
			best_lateness = lateness;
			best_schedule = Some(result.schedule);
		}
		if result.lower_bound < best_lateness && result.subproblems.is_some() {
			let new_lower_bound = max(result.lower_bound, lower_bound);
			let children = result.subproblems.unwrap();
			for child in children.into_iter() {
				subproblems.push( Reverse((
					new_lower_bound,
					child
				)));

			}
		}
	}
	best_schedule.unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CarlierNode {
	release_times: Vec<Time>,
	due_times: Vec<Time>,
}

#[derive(Debug, Clone)]
struct CarlierResult {
	schedule: JobSchedule,
	lower_bound: Time,
	subproblems: Option<[CarlierNode; 2]> // if this is None, the given schedule is optimal
}

fn carlier_iteration(
	processing_times: &[Time],
	mut release_times: Vec<Time>,
	mut due_times: Vec<Time>,
	upper_bound: Time
) -> CarlierResult
{
	let schedule = schrage(processing_times, &release_times, &due_times);
	let (a, p) = critical_path(&schedule, &due_times);
	let sched = &schedule.schedule;
	let pjob = sched[p].job;

	// find last job on the critical path with a later due date than p
	let c = match sched[a..p].iter().rev().position(|run| {
		due_times[run.job] > due_times[pjob]
	}) {
		None => return CarlierResult{  // schedule is already optimal
			lower_bound: schedule.lateness(&due_times),
			schedule,
			subproblems: None
		},
		Some(i) => p - 1 - i,
	};

	let crit_set = c+1..=p;
	let cjob = sched[c].job;

	// the critical duration is the total processing time of the critical set:
	let crit_duration: Time = sched[crit_set.clone()].iter().map(|run| run.duration ).sum();
	let crit_min_release = sched[crit_set.clone()].iter()
		.map(|run| release_times[run.job]).min().unwrap();
	 // this is the latest due time among the critical set:
	let crit_max_due = due_times[pjob];
	// this is a lower bound on the maximum lateness of any schedule:
	let crit_bound = crit_duration + crit_min_release - crit_max_due;

	for i in (a..=c).chain(p+1..sched.len()) {
		let job = sched[i].job;
		if processing_times[job] > upper_bound - crit_bound {
			// this job cannot be scheduled inside the critical set

			if release_times[job] + processing_times[job] + crit_duration 
				> upper_bound + crit_max_due
			{
				// this job has to be scheduled after the critical set
				release_times[job] = max(
					release_times[job],
					crit_min_release + crit_duration
				);
			} else if crit_min_release + crit_duration + processing_times[job]
				> upper_bound + due_times[job]
			{
				// this job has to be scheduled before the critical set
				due_times[job] = min(
					due_times[job],
					crit_max_due - crit_duration
				);
			}
		}
	}

	let lower_bound = max(
		// lower bound for c+1..p
		crit_bound,
		// lower bound for c..p
		crit_duration + min(crit_min_release, release_times[cjob]) - due_times[cjob]
	);

	// subproblem where we force c to be processed before all of crit_set:
	let mut subproblem1 = CarlierNode {
		release_times: release_times.clone(),
		due_times: due_times.clone(),
	};
	// force c before a..p:
	subproblem1.due_times[cjob] = min(due_times[cjob], due_times[pjob] - crit_duration);

	// subproblem where we force c to be processed after all of crit_set:
	let mut subproblem2 = CarlierNode {
		release_times,
		due_times,
	};
	// force c after a..p:
	subproblem2.release_times[cjob] = max(
		subproblem2.release_times[cjob],
		crit_min_release + crit_duration
	);
	CarlierResult{
		schedule,
		lower_bound,
		subproblems: Some([subproblem1, subproblem2])
	}
}


/// Returns (a, b) such that the critical path is formed 
/// by schedule[a] up to (including) schedule[b]
fn critical_path(schedule: &JobSchedule, due_times: &[Time]) -> (usize, usize) {
	let schedule = &schedule.schedule;
	let latenesses = schedule.iter().enumerate().map(
		|(i, JobRun{ time: t, job, duration: d })|
		(i, t + d - due_times[*job])
	);
	// p is the index of the job of maximum lateness
	let (p, _) = latenesses.max_by_key(
		|(_, lateness)| *lateness
	).expect("job list is empty");

	// find last job a <= p which had idle time before it
	let a = (1..=p).rev().find(|&i| {
		schedule[i].time > schedule[i-1].time + schedule[i-1].duration
	}).unwrap_or(0);
	(a, p)
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
	fn test_schrage_1() {
		let (p, r, d) = example_1();
		let expected_result = JobSchedule::from_order_durations_releasetimes(
			&vec![5, 0, 1, 3, 2, 6, 4],
			&p,
			&r
		);
		let result = schrage(&p, &r, &d);
		assert_eq!(result, expected_result);
	}

	#[test]
	fn test_edd_preemptive_1() {
		let (p, r, d) = example_1();
		let expected_result = JobSchedule{
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

	#[test]
	fn test_critical_path() {
		let (p, r, d) = example_1();
		let schedule = JobSchedule::from_durations_releasetimes(&p, &r);
		assert_eq!(critical_path(&schedule, &d), (0, 5));
	}

	fn example_2() -> (Vec<Time>, Vec<Time>, Vec<Time>) {
		(
			//    0    1    2    3   4    5   6 
			vec![ 5,   6,   7,   4,  3,   6,  2], // processing
			vec![10,  13,  11,  20, 30,   0, 30], // release
			vec![-7, -26, -24, -21, -8, -17,  0] // due
		)
	}

	#[test]
	fn test_critical_path_2() {
		let (p, r, d) = example_2();
		let schedule = JobSchedule::from_order_durations_releasetimes(
			&vec![5, 0, 1, 2, 3, 4, 6],
			&p,
			&r
		);
		assert_eq!(critical_path(&schedule, &d), (1, 4));
	}

	#[test]
	fn test_schrage_2() {
		let (p, r, d) = example_2();
		let schedule = schrage(&p, &r, &d);
		let expected_result = JobSchedule::from_order_durations_releasetimes(
			&vec![5, 0, 1, 2, 3, 4, 6],
			&p,
			&r
		);
		assert_eq!(schedule, expected_result);
	}

	#[test]
	fn test_carlier_example_2() {
		let (p, r, d) = example_2();
		let schedule = carlier(&p, &r, &d);
		let expected_result = JobSchedule::from_order_durations_releasetimes(
			&vec![5, 2, 1, 3, 0, 4, 6],
			&p,
			&r
		);
		assert_eq!(schedule, expected_result);
	}

	fn example_3() -> (Vec<Time>, Vec<Time>, Vec<Time>) {
		(
			//    0    1    2    3    4    5    6    7    8    9
			vec![ 4,   2,   5,   6,   3,   9,   2,   4,   1,   3], // processing
			vec![20,  25,  38,  12,  24,   4,  21,   6,  37,  20], // release
			vec![35,  34,  44,  32,  27,  25,  29,  31,  40,  44]  // due
		)
	}

	#[test]
	fn test_carlier_example_3() {
		let (p, r, d) = example_3();
		let schedule = carlier(&p, &r, &d);
		println!("{}", schedule);
		assert_eq!(schedule.lateness(&d), 0);
	}
}
