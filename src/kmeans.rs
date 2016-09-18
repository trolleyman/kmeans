use std::collections::HashSet;
use std::io::{self, Write};
use std::ops;
use std::time::{Instant, Duration};

use cg::prelude::*;
use cg::BaseFloat;

use rand::{thread_rng, Rng};

const MAX_STEPS: usize = 64;

// Performs the kmeans algorithm
//   - original_data: the data to be cluster identified
//   - k: the number of clusters to be identified
//   - iter: the number of iterations of the kmeans algorithm to perform, and then take the best fitting option from
// 
// Returns a tuple of
//   - The means of the clusters
//   - The data, with the index of the cluster that it belongs to
//   - The total score of the cluster arrangement
pub fn kmeans<T, F>(original_data: &[T], k: usize, iter: usize) -> (Vec<T>, Vec<(usize, T)>, F) where
		F: BaseFloat + ::std::fmt::Display,
		T: Copy + Zero + MetricSpace<Metric = F> + ops::AddAssign + ops::Sub + ops::Div<F, Output=T> {
	
	if k == 0 || k > original_data.len() {
		panic!("kmeans: invalid k: 0 < k ({}) < data length ({})", k, original_data.len());
	}
	
	// Do 16 iterations of the kmeans algorithm with 16 different random starting positions, and choose the best.
	let mut best_means = None;
	let mut best_data  = None;
	let mut best_score = None;
	
	let mut mean_indices = HashSet::with_capacity(k);
	let mut means = Vec::with_capacity(k);
	let mut data = original_data.iter().map(|&p| (0, p)).collect::<Vec<_>>();
	let mut sums = Vec::with_capacity(k);
	for i in 0..iter {
		let start_time = Instant::now();
		
		print!("k-means iteration {}/{}... ", i + 1, iter);
		io::stdout().flush().ok();
		// Initialize clusters
		mean_indices.clear();
		means.clear();
		for _ in 0..k {
			let mut i = thread_rng().gen_range(0, data.len());
			while mean_indices.contains(&i) {
				i = thread_rng().gen_range(0, data.len());
			}
			let (_, m) = data[i];
			mean_indices.insert(i);
			means.push(m);
		}
		assign_data_to_clusters(&means, &mut data);
		
		// Now perform an iteration
		let score = kmeans_iter(&mut means, &mut data, &mut sums);
		
		let end_time = Instant::now();
		let duration: Duration = end_time - start_time;
		println!("({:.2} secs) score: {}", duration.as_secs() as f64 + (duration.subsec_nanos() as f64 / 1_000_000_000.0), score);
		
		match best_score {
			Some(x) if !(score < x) => { // If current score isn't better than the best
				continue;
			}
			_ => {}
		}
		
		best_means = Some(means.clone());
		best_data  = Some(data.clone());
		best_score = Some(score);
	}
	let best_means = best_means.unwrap();
	let best_data  = best_data .unwrap();
	let best_score = best_score.unwrap();
	
	(best_means, best_data, best_score)
}

// Perform the kmeans algorithm for MAX_STEPS steps, or until the configuration reaches a local optimum. Returns the score of the current cluster positions.
fn kmeans_iter<T, F>(means: &mut Vec<T>, data: &mut Vec<(usize, T)>, sums: &mut Vec<(F, T)>) -> F where
		F: BaseFloat,
		T: Copy + Zero + MetricSpace<Metric = F> + ops::AddAssign + ops::Sub + ops::Div<F, Output=T> {
	
	// Now for a certain number of steps
	sums.clear();
	for _ in 0..means.len() {
		sums.push((F::zero(), T::zero()));
	}
	
	let threshold: F = F::from(0.0000001).unwrap();
	for _ in 0..MAX_STEPS {
		// Sum up all points in each cluster
		for i in 0..sums.len() {
			sums[i] = (F::zero(), T::zero());
		}
		for &(i, p) in data.iter() {
			sums[i].0 += F::one();
			sums[i].1 += p;
		}
		
		// Get new mean of cluster
		let mut skip = true;
		for i in 0..means.len() {
			let new_m = sums[i].1 / sums[i].0;
			if means[i].distance2(new_m).abs() >= threshold {
				skip = false;
			}
			means[i] = new_m;
		}
		// Reassign data points to clusters
		assign_data_to_clusters(&*means, data);
		
		if skip {
			break;
		}
	}
	
	// Return the score of the clusters
	score_clusters(&means, &data)
}

// Assigns data to the nearest mean.
fn assign_data_to_clusters<T, F>(means: &[T], data: &mut Vec<(usize, T)>) where
		F: BaseFloat,
		T: Copy + MetricSpace<Metric = F> {
	
	for i in 0..data.len() {
		let p = data[i].1;
		// Get min distance cluster
		let mut min_j = 0;
		let mut min_dist2 = means[0].distance2(p);
		for j in 1..means.len() {
			let dist2 = means[j].distance2(p);
			if dist2 < min_dist2 {
				min_j = j;
				min_dist2 = dist2;
			}
		}
		// Assign to cluster
		data[i].0 = min_j;
	}
}

// Assigns a score to the clusters. Larger the score, the worse the cluster positions.
fn score_clusters<T, F>(means: &[T], data: &[(usize, T)]) -> F where
		F: BaseFloat,
		T: Copy + MetricSpace<Metric = F> {
	
	let mut score = F::zero();
	
	// Sum up distances squared, as this will only be used for comparison
	for &(i, p) in data.iter() {
		score += means[i].distance2(p);
	}
	
	score
}

#[cfg(test)]
mod test {
	#[test]
	fn test_three_groups() {
		use cg::vec2;
		
		fn g(min: f32, max: f32) -> f32 {
			rand::thread_rng().gen_range(min, max)
		}
		let mut ps = vec![];
		
		const N: usize = 100;
		// Group 1
		for _ in 0..N {
			ps.push(vec2(g(9.0, 11.0), g(9.0, 11.0)))
		}
		// Group 2
		for _ in 0..N {
			ps.push(vec2(g(-1.0, 1.0), g(-1.0, 1.0)))
		}
		
		// Group 3
		for _ in 0..N {
			ps.push(vec2(g(-10.0, -9.0), g(-1.0, 1.0)))
		}
		
		let (means, data, score) = kmeans(&ps, 3);
		
		for (i, m) in means.iter().enumerate() {
			println!("{}: mean: [{}, {}]", i, m.x, m.y);
		}
		println!("total score: {}", score);
	}
}