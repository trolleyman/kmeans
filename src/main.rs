extern crate cgmath as cg;
extern crate image;
extern crate rand;
pub use cg::num_traits;

mod kmeans;

use std::cmp::Ordering;
use std::env::args_os;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::exit;

use cg::vec3;

use image::{RgbImage, ImageBuffer};

fn usage() -> ! {
	writeln!(io::stderr(), "Usage: kmeans <in-path> [k] [out-path]").ok();
	exit(1);
}

struct ParsedArgs {
	in_path: PathBuf,
	out_path: PathBuf,
	k: usize
}
impl ParsedArgs {
	pub fn new() -> ParsedArgs {
		let args: Vec<_> = args_os().collect();
		
		if args.len() < 2 || args.len() > 4 {
			usage();
		}
		
		let in_path = PathBuf::from(args[1].clone());
		let mut out_path = None;
		let mut k = None;
		for i in 2..args.len() {
			let s = args[i].clone();
			match s.clone().into_string().ok().and_then(|s| s.parse::<usize>().ok()) {
				Some(x) if k.is_none() => {
					k = Some(x);
				}
				_ => {
					out_path = Some(PathBuf::from(s));
				}
			}
		}
		let k = k.unwrap_or(3);
		
		let out_path = out_path.unwrap_or_else(|| {
			let mut name = in_path.file_stem().unwrap().to_os_string();
			name.push(format!("-{}", k));
			if let Some(ext) = in_path.extension() {
				name.push(".");
				name.push(ext);
			}
			in_path.with_file_name(name)
		});
		
		let in_path = match in_path.canonicalize() {
			Ok(p)  => p,
			Err(_) => in_path,
		};
		
		let out_path = match out_path.canonicalize() {
			Ok(p)  => p,
			Err(_) => out_path,
		};
		
		ParsedArgs{
			in_path: in_path,
			out_path: out_path,
			k: k
		}
	}
}

fn main() {
	let args = ParsedArgs::new();
	
	println!("Reading image from '{}'...", args.in_path.display());
	
	let img = match image::open(&args.in_path) {
		Ok(i) => i,
		Err(e) => {
			writeln!(io::stderr(), "Error: {}", e).ok();
			exit(2);
		}
	};
	
	let img = img.to_rgb();
	let w = img.width();
	let h = img.height();
	let data_bytes = img.into_raw();
	
	let mut data = Vec::with_capacity(data_bytes.len() / 3);
	for i in 0..data_bytes.len() / 3 {
		data.push(vec3(
			data_bytes[i * 3    ] as f64,
			data_bytes[i * 3 + 1] as f64,
			data_bytes[i * 3 + 2] as f64
		));
	}
	
	::std::mem::drop(data_bytes);
	
	println!("Sorting {} data points into {} clusters using k-means algorithm", data.len(), args.k);
	let (means, data, loss) = kmeans::kmeans(&data, args.k, 6);
	println!("final loss: {}", loss);
	
	let mut data_bytes = Vec::with_capacity(data.len() * 3);
	unsafe {
		// This is fine as capacity is data.len() * 3
		data_bytes.set_len(data.len() * 3);
	}
	
	let mut colours = Vec::with_capacity(means.len());
	for m in means.iter() {
		colours.push(vec3(m.x as u8, m.y as u8, m.z as u8));
	}
	
	for i in 0..data.len() {
		let j = data[i].0;
		data_bytes[i * 3    ] = colours[j].x;
		data_bytes[i * 3 + 1] = colours[j].y;
		data_bytes[i * 3 + 2] = colours[j].z;
	}
	println!("{} colour(s) selected:", args.k);
	colours.sort_by(|a, b| {
		if a.x != b.x {
			a.x.cmp(&b.x)
		} else if a.y != b.y {
			a.y.cmp(&b.y)
		} else if a.z != b.z {
			a.z.cmp(&b.z)
		} else {
			Ordering::Equal
		}
	});
	for &c in colours.iter() {
		println!("R: {:3}, G: {:3}, B: {:3}", c.x, c.y, c.z);
	}
	
	println!("Saving image to '{}'...", args.out_path.display());
	let img: RgbImage = ImageBuffer::from_raw(w, h, data_bytes).unwrap();
	match img.save(&args.out_path) {
		Ok(()) => {},
		Err(e) => {
			writeln!(io::stderr(), "{}", e).ok();
			exit(3);
		}
	}
}
