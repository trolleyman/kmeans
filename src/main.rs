extern crate cgmath as cg;
extern crate image;
extern crate rand;
pub use cg::num_traits;

mod kmeans;

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
		
		let out_path = out_path.unwrap_or_else(|| {
			let mut name = in_path.file_stem().unwrap().to_os_string();
			name.push(" (Edited).");
			name.push(in_path.extension().unwrap());
			in_path.with_file_name(name)
		});
		
		ParsedArgs{
			in_path: in_path,
			out_path: out_path,
			k: k.unwrap_or(3)
		}
	}
}

fn main() {
	let args = ParsedArgs::new();
	
	println!("Reading image from '{}'", args.in_path.display());
	
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
	let mut data_bytes = img.into_raw();
	
	let mut data = Vec::with_capacity(data_bytes.len() / 3);
	for i in 0..data_bytes.len() / 3 {
		data.push(vec3(
			data_bytes[i * 3    ] as f64,
			data_bytes[i * 3 + 1] as f64,
			data_bytes[i * 3 + 2] as f64
		));
	}
	
	println!("Running kmeans algorithm");
	let (means, data, score) = kmeans::kmeans(&data, args.k, 8);
	println!("final score: {}", score);
	for i in 0..data.len() {
		let j = data[i].0;
		data_bytes[i * 3    ] = means[j].x as u8;
		data_bytes[i * 3 + 1] = means[j].y as u8;
		data_bytes[i * 3 + 2] = means[j].z as u8;
	}
	println!("{} colour(s) selected:", args.k);
	for &m in means.iter() {
		println!("R: {:3}, G: {:3}, B: {:3}", m.x as u8, m.y as u8, m.z as u8);
	}
	
	println!("Saving image to '{}'", args.out_path.display());
	let img: RgbImage = ImageBuffer::from_raw(w, h, data_bytes).unwrap();
	match img.save(&args.out_path) {
		Ok(()) => {},
		Err(e) => {
			writeln!(io::stderr(), "{}", e).ok();
			exit(3);
		}
	}
}
