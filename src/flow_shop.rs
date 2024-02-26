use crate::jobs::{Time, Job};

/// Optimally schedules jobs in a 2-machine flow shop to minimize makespan.
/// This function uses Johnson's algorithm and takes O(n log n) time.
/// See Johnson: "Optimal two- and three-stage production schedules with setup times included", 1954.
///
/// # Arguments
/// * processing_times: The processing time of each job on each of the two machines
///
/// # Returns
/// A permutation of the jobs (i.e. of 0..n-1) such that scheduling the jobs in this order on both machines
/// is an optimal solution ot the given F2||C_max instance.
pub fn johnson(processing_times: &[[Time; 2]]) -> Vec<Job> {
	let n = processing_times.len();
	let group1 = (0..n).filter(|&j| processing_times[j][0] <= processing_times[j][1]);
	let mut result : Vec<Job> = group1.collect();
	result.sort_unstable_by_key(|&j| processing_times[j][0]);
	let size1 = result.len();
	let group2 = (0..n).filter(|&j| processing_times[j][0] > processing_times[j][1]);
	result.extend(group2);
	result[size1..].sort_unstable_by_key(|&j| -processing_times[j][1]);
	result
}


#[cfg(test)]
mod tests {
	use super::*;

	fn example_1() -> Vec<[Time; 2]> {
		vec![[3, 4], [2, 1], [1, 5]]
	}

	#[test]
	fn test_johnson_example_1() {
		let result = johnson(&example_1());
		let expected = vec![2, 0, 1];
		assert_eq!(result, expected);
	}
}