use std::env;
use std::fs;
use std::path::Path;

fn main() {
	let main_font_symbols = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ААБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщьыъэюя";
	let numbers_symbols = "0123456789";
	let hex_symbols = "0123456789abcdef";
	let fonts = vec![
		("MAIN_FONT", "main-font.bmp", 5_u32, 9_u32, main_font_symbols),
		("NUMBERS", "small-numbers.bmp", 3, 5, numbers_symbols),
		("HEX", "hex-numbers.bmp", 3, 5, hex_symbols),
	];

	let mut generated_code: String = "".to_string();

	for (const_name, filepath, symb_width, symb_height, symbols) in fonts {
		//println!("cargo:rerun-if-changed={}", filepath);

		let full_img = bmp::open(filepath).unwrap_or_else(|e| {
			panic!("Failed to open: {}", e);
		});
		let pixel = |x: u32, y: u32| {
			if x < full_img.get_width() && y < full_img.get_height() {
				let p = full_img.get_pixel(x, y);
				p.r <= 16 && p.g <= 16 && p.b <= 16
			} else {
				false
			}
		};

		let mut row = 0;
		let mut col = 0;

		let mut array_code = "".to_string();
		let mut chars_code = "".to_string();

		for symbol in symbols.chars() {
			let mut symbol_look: Vec<Vec<bool>> = vec![];

			for y in 0..symb_height {
				let row: Vec<bool> = (0..symb_width).map(
					|x| pixel(col * (symb_width + 1) + x, row * (symb_height + 1) + y)
				).collect();
				symbol_look.push(row);
			}

			col += 1;
			if col * (symb_width + 1) >= full_img.get_width() {
				col = 0;
				row += 1;
			}

			let symbol = match symbol {
				'\'' => format!("\\'"),
				'"' => format!("\\\""),
				'\\' => format!("\\\\"),
				other=> format!("{}", other),
			};
			array_code.push_str(&format!("('{}', {:?}),\n", symbol, symbol_look));
			chars_code.push_str(&symbol);
		}


		let append = format!(
			"pub const {}: [(char, [[bool; {}]; {}]); {}] = [\n{}];\n\
			pub const {}_SYMBOLS: &str = \"{}\";\n",
			const_name, symb_width, symb_height, symbols.chars().count(), array_code,
			const_name, chars_code
		);

		generated_code.push_str(&append);
	}

	let out_dir = env::var_os("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("fonts_generated.rs");
	fs::write(&dest_path, generated_code).unwrap();
	println!("cargo:rerun-if-changed=build.rs");
}