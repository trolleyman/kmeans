extern crate cgmath as cg;
extern crate rand;
pub use cg::num_traits;

use cg::{BaseFloat, vec2};
use cg::prelude::*;

use rand::Rng;

fn main() {
	fn g(min: f32, max: f32) -> f32 {
		rand::thread_rng().gen_range(min, max)
	}
	let mut ps = vec![];
	// Group 1
	for _ in 0..10 {
		ps.push(vec2(g(9.0, 11.0), g(9.0, 11.0)))
	}
	// Group 2
	for _ in 0..10 {
		ps.push(vec2(g(-1.0, 1.0), g(-1.0, 1.0)))
	}
	
	// Group 3
	for _ in 0..10 {
		ps.push(vec2(g(-10.0, -9.0), g(-1.0, 1.0)))
	}
	
	let (cs, score) = kmeans(ps, 3);
	
	for (i, c) in cs.iter().enumerate() {
		println!("{}: mean: [{}, {}]", i, c.mean.x, c.mean.y);
	}
	println!("total score: {}", score);
}


pub struct Cluster<T> {
	pub mean: T,
	pub data: Vec<T>
}

const MAX_STEPS: usize = 64;

// Performs the kmeans algorithm
//   - original_data: the data to be cluster identified
//   - k: the number of clusters to be identified
//   - iter: the number of iterations of the kmeans algorithm to perform, and then take the best fitting option from
// Returns clusters and the total score
pub fn kmeans<T, F>(original_data: &[T], k: usize, iter: usize) -> (Vec<Cluster<T>>, F) where
		F: BaseFloat,
		T: Copy + Zero + MetricSpace<Metric = F> + std::ops::AddAssign + std::ops::Sub + std::ops::Div<F, Output=T> {
	
	if k == 0 || k > original_data.len() {
		panic!("kmeans: invalid k: 0 < k ({}) < data length ({})", k, original_data.len());
	}
	
	// Do 16 iterations of the kmeans algorithm with 16 different random starting positions, and choose the best.
	let mut best_means = None;
	let mut best_data  = None;
	let mut best_score = None;
	
	let mut means = Vec::with_capacity(k);
	let mut data = original_data.iter().map(|&p| (0, p)).collect::<Vec<_>>();
	let mut sums = Vec::with_capacity(k);
	for _ in 0..iter {
		// Initialize clusters
		means.clear();
		for _ in 0..k {
			let i = rand::thread_rng().gen_range(0, data.len());
			let (_, m) = data.swap_remove(i);
			means.push(m);
		}
		for &m in means.iter() {
			data.push((0, m));
		}
		assign_data_to_clusters(&means, &mut data);
		
		// Now perform an iteration
		let score = kmeans_iter(&mut means, &mut data, &mut sums);
		
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
	
	// Gather up clusters
	let mut clusters = Vec::with_capacity(best_means.len());
	for i in 0..best_means.len() {
		clusters.push(Cluster{
			mean : best_means[i],
			data : Vec::new()
		});
	}
	
	for (i, p) in best_data {
		clusters[i].data.push(p);
	}
	
	(clusters, best_score)
}

// Perform the kmeans algorithm for MAX_STEPS steps, or until the configuration reaches a local optimum. Returns the score of the current cluster positions.
fn kmeans_iter<T, F>(means: &mut Vec<T>, data: &mut Vec<(usize, T)>, sums: &mut Vec<(F, T)>) -> F where
		F: BaseFloat,
		T: Copy + Zero + MetricSpace<Metric = F> + std::ops::AddAssign + std::ops::Sub + std::ops::Div<F, Output=T> {
	
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