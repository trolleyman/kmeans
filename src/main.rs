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

fn main() {
	let args: Vec<_> = args_os().collect();
	
	if args.len() < 2 || args.len() > 3 {
		writeln!(io::stderr(), "Usage: kmeans <in-path> [out-path]").ok();
		exit(1);
	}
	
	let in_path = PathBuf::from(args[1].clone());
	
	println!("Reading image from '{}'", in_path.display());
	
	let img = match image::open(&in_path) {
		Ok(i) => i,
		Err(e) => {
			writeln!(io::stderr(), "Error: {}", e).ok();
			exit(2);
		}
	};
	
	let out_path = if args.len() == 3 {
		PathBuf::from(args[2].clone())
	} else {
		let mut name = in_path.file_stem().unwrap().to_os_string();
		name.push(" (Edited).");
		name.push(in_path.extension().unwrap());
		in_path.with_file_name(name)
	};
	
	let img = img.to_rgb();
	let w = img.width();
	let h = img.height();
	let mut data_bytes = img.into_raw();
	
	let mut data = Vec::with_capacity(data_bytes.len() / 3);
	for i in 0..data_bytes.len() / 3 {
		data.push(vec3(
			data_bytes[i * 3    ] as f32,
			data_bytes[i * 3 + 1] as f32,
			data_bytes[i * 3 + 2] as f32
		));
	}
	
	println!("Running kmeans algorithm");
	let (means, data, score) = kmeans::kmeans(&data, 3, 8);
	println!("final score: {}", score);
	for i in 0..data.len() {
		let j = data[i].0;
		data_bytes[i * 3    ] = means[j].x as u8;
		data_bytes[i * 3 + 1] = means[j].y as u8;
		data_bytes[i * 3 + 2] = means[j].z as u8;
	}
	
	println!("Saving image to '{}'", out_path.display());
	let img: RgbImage = ImageBuffer::from_raw(w, h, data_bytes).unwrap();
	match img.save(out_path) {
		Ok(()) => {},
		Err(e) => {
			writeln!(io::stderr(), "{}", e).ok();
			exit(3);
		}
	}
}
