use std::cmp::max;

use crate::{Time, Job, MachineSchedule, MultiMachineSchedule, JobRun, Machine};


/// Makespan-minimization heuristic for scheduling on multiple unrelated machines with precedence constraints,
/// i.e. for R|prec|C_max.
/// The heuristic always selects the available job whose processing time has the highest variance 
/// among the machines. This job is then greedily scheduled on the fastest machine currently avaiable.
/// The running time is in O(jobs^2).
///
/// See Liu & Yang "A heuristic serial schedule algorithm for unrelated parallel machine scheduling with
/// precedence constraints" (doi:10.4304/jsw.6.6.1146-1153)
///
/// # Arguments
/// * `ptimes`: Job processing times, where `ptimes[i][j]` is the time taken by machine `i` to process job `j`.
/// * `predecessor`: Job predecessors, where `predecessor[i]` are the jobs that need to be completed before job `i` can be started.
/// 
/// # Returns
/// The resulting schedule.
///
pub fn serial_schedule_heuristic(
	ptimes: &[Vec<Time>],
	predecessor: Vec<Vec<Job>>
) -> MultiMachineSchedule
{
	let m = ptimes.len(); // number of machines
	if m == 0 {
		return MultiMachineSchedule::new();
	}
	let n = ptimes[0].len(); // number of jobs
	let mut schedules = vec![MachineSchedule::new(); m];
	if n == 0 {
		return MultiMachineSchedule{ machine_schedules: schedules }
	}
	let mut time = 0;
	let mut pg = PrecedenceGraph::new(predecessor);
	let mut machines_busy_until : Vec<Time> = vec![0; m];
	let mut completion_times : Vec<(Time, Job)> = Vec::new();
	for counter in 0.. {
		let idle_machines : Vec<_> = machines_busy_until.iter().enumerate()
			.filter(|&(_, &t)| t <= time)
			.map(|(i, _)| i)
			.collect();
		let (machine, job, duration) = serial_schedule_heuristic_pick_next(
			ptimes,
			&idle_machines,
			pg.available_jobs()
		);
		schedules[machine].schedule.push(
			JobRun{
				time,
				job,
				duration
			}
		);
		if counter == n-1 {
			break; // all jobs scheduled
		}
		pg.mark_job_running(job);
		completion_times.push((time + duration, job));
		machines_busy_until[machine] = time + duration;
		// wait for next avaiable machine
		time = max(time, *machines_busy_until.iter().min().unwrap());
		// mark completed jobs
		completion_times.retain(|&(t, j)| {
			if t <= time {
				pg.mark_job_completed(j);
			}
			t > time
		});
		while pg.available_jobs().is_empty() {
			// wait for next avaiable machine
			time = *machines_busy_until.iter().filter(|&&t| t > time).min().unwrap();
			// mark completed jobs
			completion_times.retain(|&(t, j)| {
				if t <= time {
					pg.mark_job_completed(j);
				}
				t > time
			});
		}
	}
	MultiMachineSchedule{
		machine_schedules: schedules
	}
}

fn serial_schedule_heuristic_pick_next(
	ptimes: &[Vec<Time>],
	idle_machines: &[Machine],
	available_jobs: &[Job],
) -> (Machine, Job, Time)
{
	let machine;
	let job;
	let duration : Time;
	assert!(!idle_machines.is_empty());
	if idle_machines.len() == 1 {
		// schedule the shortest job
		machine = idle_machines[0];
		(duration, job) = available_jobs.iter().map(|&j|
			(ptimes[machine][j], j)
		).min().unwrap();
	} else {
		// select the job with the highest processing time variance:
		(job, _) = available_jobs.iter().map(|&j| {
				// mean processing time:
				let mean = 
					ptimes.iter().map(|p| p[j] as f32).sum::<f32>()
					/ idle_machines.len() as f32;
				let variance = ptimes.iter().map(|p| 
					(p[j] as f32 - mean)*(p[j] as f32 - mean)
				).sum::<f32>() / idle_machines.len() as f32;
				(j, variance)
		}).max_by(
			|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap()
		).unwrap();
		// select the machine that's fastest for that job:
		(machine, duration) = idle_machines.iter()
			.map(|&i| (i, ptimes[i][job]) )
			.min_by_key(|&(_, p)| p)
			.unwrap();
	}
	(machine, job, duration)
}


struct PrecedenceGraph {
	available: Vec<Job>,
	predecessor: Vec<Vec<Job>>,
}

impl PrecedenceGraph {
	pub fn available_jobs(&self) -> &[Job] {
		&self.available
	}

	/// Marks the given job as completed,
	/// thus removing it as a precondition for all other jobs.
	pub fn mark_job_completed(&mut self, job: Job) {
		self.mark_job_running(job);
		// remove the job from every other job's precedence list
		for (i, pr) in self.predecessor.iter_mut().enumerate() {
			if i != job && !pr.is_empty() {
				if let Some(pos) = pr.iter().position(|&j| j == job) {
					pr.swap_remove(pos);
				}
				if pr.is_empty() {
					self.available.push(i);
				}
			}
		}
	}

	/// Marks the given job as running,
	/// thus removing it from the list of available jobs now and forever.
	pub fn mark_job_running(&mut self, job: Job) {
		if let Some(index) = self.available.iter().position(|&j| j == job) {
			self.available.swap_remove(index);
		}
		// set job to be its own precedence to prevent it ever becoming avaiable again
		self.predecessor[job].clear();
		self.predecessor[job].push(job);
	}

	pub fn new(predecessor: Vec<Vec<Job>>) -> PrecedenceGraph {
		let available = predecessor.iter().enumerate().filter(
			|(_, p)| p.is_empty()
		).map(|(i, _)| i).collect();
		PrecedenceGraph {
			available,
			predecessor,
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_precedence_graph() {
		let prec = vec![
			vec![1],
			vec![],
			vec![1],
			vec![0, 2],
			vec![2],
		];
		let mut pg = PrecedenceGraph::new(prec);
		assert_eq!(pg.available_jobs(), vec![1]);
		
		pg.mark_job_completed(1);
		let mut result = pg.available_jobs().to_vec();
		result.sort();
		assert_eq!(result, vec![0, 2]);

		pg.mark_job_completed(2);
		let mut result = pg.available_jobs().to_vec();
		result.sort();
		assert_eq!(result, vec![0, 4]);

		pg.mark_job_completed(0);
		let mut result = pg.available_jobs().to_vec();
		result.sort();
		assert_eq!(result, vec![3, 4]);
	}

	#[test]
	fn test_serial_schedule_heuristic() {
		let p = vec![
			vec![4, 4, 9, 2, 3, 2], // processing times on machine 0
			vec![6, 4, 3, 3, 7, 5], // processing times on machine 2
		];
		let prec = vec![
			vec![3], // jobs required for job 0
			vec![0, 5],
			vec![4],
			vec![],
			vec![],
			vec![],
		];
		let schedule = serial_schedule_heuristic(&p, prec);
		// optimal makespan is actually 12 
		// (run jobs 3, 5, 4, 1 on machine 0)
		assert!(schedule.makespan() <= 13);
	}

	#[test]
	fn test_serial_schedule_heuristic_2() {
		// this is the example given in doi:10.4304/jsw.6.6.1146-1153
		let p = vec![
			vec![3, 4, 8, 2,  5, 9, 3],
			vec![9, 5, 2, 6, 10, 4, 8],
		];
		let prec = vec![
			vec![],
			vec![],
			vec![0],
			vec![],
			vec![],
			vec![1],
			vec![2],
		];
		let schedule = serial_schedule_heuristic(&p, prec);
		assert_eq!(schedule.makespan(), 13);
	}
}