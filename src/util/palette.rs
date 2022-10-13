use crate::util::Point;

const INPUT_COLORS: [(i32, i32, i32); 8] = [
	(10,  62, 226), 	// 0A3EE2
	(208, 37, 37), 		// D02525
	(117, 20, 237),	 	// 7514ED
	(10,  62, 226), 	// CF11D2
	(76,  111, 227),	// 4C6FE3
	(240, 103, 103),	// F06767
	(174, 121, 240),	// AE79F0
	(238, 123, 240),	// EE7BF0
];

pub const OUTPUT_COLORS: [(i32, i32, i32); 8] = [
	(25,  231, 83),		// 19E753
	(160, 234, 0),		// A0EA00
	(44,  230, 230),	// 2CE6E6
	(226, 219, 19),		// E2DB13
	(104, 255, 136),	// 68FF88
	(203, 246, 111),	// CBF66F
	(126, 237, 237),	// 7EEDED
	(245, 240, 113),	// F5F071
];

pub fn color_to_string(r: i32, g: i32, b: i32) -> String {
	let r = r.min(255).max(0);
	let g = g.min(255).max(0);
	let b = b.min(255).max(0);

	format!("{:02x}{:02x}{:02x}", r, g, b)
}

pub fn input_color(input_id: u32, point: Point) -> String {
	let (r, g, b) = INPUT_COLORS[(input_id as usize) % INPUT_COLORS.len()];

	// I just love to have some color fluctuations
	let dr = 80;
	let dg = 80;
	let db = 80;

	let r = r + (((*point.x() as f32) / 10.0).sin() * (dr as f32)).round() as i32;
	let g = g + (((*point.y() as f32) / 10.0).sin() * (dg as f32)).round() as i32;
	let b = b + (((*point.z() as f32) / 10.0).sin() * (db as f32)).round() as i32;

	color_to_string(r, g, b)
}

pub fn output_color(output_id: u32, point: Point) -> String {
	let (r, g, b) = OUTPUT_COLORS[(output_id as usize) % OUTPUT_COLORS.len()];

	// I just love to have some color fluctuations
	let dr = 80;
	let dg = 80;
	let db = 80;

	let r = r + (((*point.x() as f32) / 10.0).sin() * (dr as f32)).round() as i32;
	let g = g + (((*point.y() as f32) / 10.0).sin() * (dg as f32)).round() as i32;
	let b = b + (((*point.z() as f32) / 10.0).sin() * (db as f32)).round() as i32;

	color_to_string(r, g, b)
}