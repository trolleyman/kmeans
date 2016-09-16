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
	
	let cs = kmeans(ps, 3);
	for (i, c) in cs.iter().enumerate() {
		println!("{}: [{}, {}]", i, c.mean.x, c.mean.y);
	}
}

struct Cluster<T> {
	pub mean: T,
	pub data: Vec<T>
}

fn kmeans<T, F>(mut data: Vec<T>, k: usize) -> Vec<Cluster<T>> where
		F: BaseFloat,
		T: Copy + Zero + MetricSpace<Metric = F> + std::ops::AddAssign + std::ops::Sub + std::ops::Div<F, Output=T> {
	
	if k == 0 || k > data.len() {
		panic!("kmeans: invalid k: 0 < k ({}) < data length ({})", k, data.len());
	}
	
	// Initialize clusters
	let mut means = Vec::new();
	for _ in 0..k {
		let i = rand::thread_rng().gen_range(0, data.len());
		let m = data.swap_remove(i);
		means.push(m);
	}
	for m in means.iter().cloned() {
		data.push(m);
	}
	let mut data = data.into_iter().map(|p| (0, p)).collect();
	
	assign_clusters(&means, &mut data);
	
	// Now for a certain number of steps
	let mut sums = Vec::with_capacity(means.len());
	for _ in 0..means.len() {
		sums.push((F::zero(), T::zero()));
	}
	
	let threshold: F = F::from(0.0000001).unwrap();
	for _ in 0..50 {
		// Sum up all points in each cluster
		for i in 0..sums.len() {
			sums[i] = (F::zero(), T::zero());
		}
		for &(i, p) in data.iter() {
			sums[i].0 += F::one();
			sums[i].1 += p;
		}
		
		let mut skip = true;
		for i in 0..means.len() {
			let new_m = sums[i].1 / sums[i].0;
			if means[i].distance2(new_m).abs() >= threshold {
				skip = false;
			}
			means[i] = new_m;
		}
		assign_clusters(&means, &mut data);
		
		if skip {
			break;
		}
	}
	
	// Gather up clusters
	let mut clusters = Vec::with_capacity(means.len());
	for m in means {
		clusters.push(Cluster{ mean:m, data:Vec::new() });
	}
	
	for (i, p) in data {
		clusters[i].data.push(p);
	}
	
	clusters
}

fn assign_clusters<T, F>(means: &[T], data: &mut Vec<(usize, T)>) where
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
