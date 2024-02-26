use crate::{Time, Job};


/// Optimally schedules jobs in a 2-machine flow shop to minimize makespan.
/// This function uses Johnson's algorithm and takes O(n log n) time.
/// See Johnson: "Optimal two- and three-stage production schedules with setup times included", 1954.
///
/// # Arguments
/// * ptimes: The processing times, where ptimes[i][j] is the time taken by machine i for job j
///
/// # Returns
/// A permutation of the jobs (i.e. of 0..n-1) such that scheduling the jobs in this order on both machines
/// is an optimal solution ot the given F2||C_max instance.
pub fn johnson(ptimes: &[Vec<Time>]) -> Vec<Job> {
	assert!(ptimes.len() == 2, "Instance must have exactly 2 machines");
	let n = ptimes[0].len();
	let mut result : Vec<Job> = (0..n).collect();
	let num1 = partition_in_place(&mut result, 
		|&j| ptimes[0][j] <= ptimes[1][j]
	);
	result[..num1].sort_unstable_by_key( |&j|  ptimes[0][j] );
	result[num1..].sort_unstable_by_key( |&j| -ptimes[1][j] );
	result
}


/// Reorders a vector in place according to a predicate function,
/// such that all items satisfying the predicate come before any other item.
///
/// # Returns
/// The number of items that satisfy the predicate
fn partition_in_place<T, F>(container: &mut Vec<T>, mut predicate: F) -> usize
where
	F: FnMut(&T) -> bool,
{
	let mut i1 = 0;
	let mut i2 = container.len() - 1;
	loop {
		while i1 < container.len() - 1 && predicate(&container[i1]) {
			i1 += 1;
		}
		while i2 > 0 && !predicate(&container[i2]) {
			i2 -= 1;
		}
		if i1 < i2 {
			container.swap(i1, i2);
		}
		else {
			return i1;
		}
	}
}


/// Produces a heuristic schedule for a flow shop instance that aims to minimize makespan (i.e. for F||C_max)
/// This function uses Dannebring's algorithm and takes O(n log n) time.
/// See Dannenbring: "An evaluation of flow shop sequencing heuristics", 1977
///
/// # Arguments
/// * ptimes: The processing times where `ptimes[i][j]` is the time needed by machine i for job j.
///
/// # Returns
/// A permutation of the jobs (i.e. of 0..n-1) such that scheduling the jobs in this order on both machines yields the proposed schedule.
pub fn dannenbring(ptimes: &[Vec<Time>]) -> Vec<Job> {
	let m = ptimes.len(); // number of machines
	if m == 0 {
		return Vec::new()
	}
	let n = ptimes[0].len(); // number of jobs
	let weights1 : Vec<_> = (0..n).map(
		|j| (0..m).map( |i| ((m-i) as isize)*ptimes[i][j] ).sum()
	).collect();
	let weights2 : Vec<_> = (0..n).map(
		|j| (0..m).map( |i| ((i+1) as isize)*ptimes[i][j] ).sum()
	).collect();
	johnson( &[weights1, weights2] )
}


#[cfg(test)]
mod tests {
	use super::*;
	use crate::MultiMachineSchedule;

	fn example_1() -> Vec<Vec<Time>> {
		vec![vec![3, 2, 1], vec![4, 1, 5]]
	}

	#[test]
	fn test_johnson_example_1() {
		let result = johnson(&example_1());
		let expected = vec![2, 0, 1];
		assert_eq!(result, expected);
	}

	#[test]
	fn test_partition_in_place() {
		let mut container = vec![3, 4, 7, 1, 0, 2, 0, 4];
		let is_even : fn(&i32) -> _ = |x| x % 2 == 0;
		let k = partition_in_place(&mut container, is_even);
		container[..k].sort_unstable();
		container[k..].sort_unstable();
		assert_eq!(container, vec![0, 0, 2, 4, 4, 1, 3, 7]);
	}

	fn example_2() -> Vec<Vec<Time>> {
		vec![
			vec![3, 4, 10],
			vec![11, 1, 5],
			vec![7, 9, 13],
			vec![10, 12, 2],
		]
		// the optimal flow shop schedule is given by the permutation 0, 3, 2, 1
	}

	#[test]
	fn test_dannenbring_example_2() {
		let ptimes = example_2();
		let result = dannenbring(&ptimes);
		let schedule = MultiMachineSchedule::from_order_ptimes(&result, &ptimes);
		assert!(schedule.makespan() <= 40);
		assert!(schedule.makespan() >= 39); // this is the optimal solution
	}
}