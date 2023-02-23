use crate::jobs::{Time, Job, JobSchedule};


/// Makespan-minimization heuristic for scheduling on multiple unrelated machines with precedents constraints.
/// See Liu & Yang "A heuristic serial schedule algorithm for unrelated parallel machine scheduling with
/// precedence constraints" (doi:10.4304/jsw.6.6.1146-1153)
pub fn serial_schedule_heuristic(
	processing_times: &[Vec<Time>],
	precedents: &[Vec<Job>]
) -> Vec<JobSchedule>
{
	let m = processing_times.len(); // number of machines
	let n = processing_times[0].len(); // number of jobs
	for u in 0..=n {
		// todo
	}
	unimplemented!()
}


struct PrecedenceGraph {
	available: Vec<Job>,
	precedents: Vec<Vec<Job>>,
}

impl PrecedenceGraph {
	pub fn available_jobs(self: &Self) -> &[Job] {
		&self.available
	}

	/// Marks the given job as completed,
	/// thus removing it as a precondition for all other jobs.
	/// The job will no longer appear in the list of available jobs at any future point.
	pub fn mark_job_completed(self: &mut Self, job: Job) {
		// remove the job from every other job's precedence list
		for (i, mut pr) in self.precedents.iter_mut().enumerate() {
			if i != job && !pr.is_empty() {
				if let Some(pos) = pr.iter().position(|&j| j == job) {
					pr.swap_remove(pos);
				};
				if pr.is_empty() {
					self.available.push(i);
				}
			}
		}
		if let Some(index) = self.available.iter().position(|&j| j == job) {
			self.available.swap_remove(index);
		}
		self.precedents[job].clear();
		// add job as its own precedence to prevent it ever becoming avaiable again
		self.precedents[job].push(job);
	}

	pub fn new(precedents: Vec<Vec<Job>>) -> PrecedenceGraph {
		let available = precedents.iter().enumerate().filter(
			|(i, p)| p.is_empty()
		).map(|(i, _)| i).collect();
		PrecedenceGraph {
			available,
			precedents,
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
		let schedules = serial_schedule_heuristic(&p, &prec);
		assert_eq!(schedules.iter().map(|s| s.makespan()).max().unwrap(), 12);
	}

}