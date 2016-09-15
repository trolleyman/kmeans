extern crate rand;

use rand::Rng;

fn main() {
	fn g(min: f32, max: f32) -> f32 {
		rand::thread_rng().gen_range(min, max)
	}
	let mut ps = vec![];
	// Group 1
	for _ in 0..10 {
		ps.push(Point{x:g(9.0, 11.0), y:g(9.0, 11.0)})
	}
	// Group 2
	for _ in 0..10 {
		ps.push(Point{x:g(-1.0, 1.0), y:g(-1.0, 1.0)})
	}
	
	let cs = kmeans(ps, 2);
	for c in cs.iter() {
		println!("c: [{}, {}], :: {:?}", c.mean.x, c.mean.y, c.data);
	}
}

#[derive(PartialEq, Copy, Clone, Debug)]
struct Point {
	x: f32,
	y: f32
}

struct Cluster {
	mean: Point,
	data: Vec<Point>
}

#[derive(PartialEq, PartialOrd)]
struct NotNaN(f32);
impl std::cmp::Eq for NotNaN {}
impl std::cmp::Ord for NotNaN {
	fn cmp(&self, other: &NotNaN) -> std::cmp::Ordering {
		self.partial_cmp(other).unwrap()
	}
}

fn kmeans(mut data: Vec<Point>, k: u32) -> Vec<Cluster> {
	if k == 0 || k as usize > data.len() {
		panic!("Invalid k");
	}
	
	// Initialize clusters
	let mut clusters = Vec::new();
	for _ in 0..k {
		let i = rand::thread_rng().gen_range(0, data.len());
		let cluster_mean = data.swap_remove(i);
		let cluster = Cluster { mean: cluster_mean, data: Vec::new() };
		clusters.push(cluster);
	}
	
	// Add all data to first cluster, then equalize
	clusters[0].data = data;
	equalize_clusters(&mut clusters);
	
	// Now for a certain number of steps
	for _ in 0..20 {
		for cluster in clusters.iter_mut() {
			let new_mean = mean(&cluster.data).unwrap_or(cluster.mean);
			cluster.mean = new_mean;
		}
		equalize_clusters(&mut clusters);
	}
	
	clusters
}

fn equalize_clusters(cs1: &mut Vec<Cluster>) {
	let mut cs2 = Vec::new();
	for c in cs1.iter() {
		cs2.push(Cluster { mean: c.mean, data: Vec::new() });
	}
	// For each point...
	for p in cs1.drain(..).flat_map(|mut c| c.data.into_iter()) {
		// Add point to closest cluster
		cs2.iter_mut()
		   .min_by_key(|c| NotNaN(dist_sq(c.mean, p)))
		   .unwrap()
		   .data.push(p);
	}
	*cs1 = cs2;
}

fn dist_sq(a: Point, b: Point) -> f32 {
	let dx = a.x - b.x;
	let dy = a.y - b.y;
	dx * dx + dy * dy
}

/// Get the mean point of some points
fn mean(data: &[Point]) -> Option<Point> {
	if data.len() == 0 {
		None
	} else {
		let mut sum_x = data[0].x;
		let mut sum_y = data[0].y;
		for i in 1..data.len() {
			sum_x += data[i].x;
			sum_y += data[i].y;
		}
		sum_x /= data.len() as f32;
		sum_y /= data.len() as f32;
		Some(Point{ x:sum_x, y:sum_y })
	}
}
