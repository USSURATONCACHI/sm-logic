use std::collections::HashMap;
use crate::combiner::{Combiner, CompileError};
use crate::presets::{binary_selector_compact};
use crate::scheme::Scheme;
use crate::shape::vanilla::{BlockBody, BlockType};
use crate::shape::vanilla::GateMode::{AND, OR};
use crate::util::Rot;
include!(concat!(env!("OUT_DIR"), "/fonts_generated.rs"));

#[derive(Debug, Clone)]
pub struct Font {
	chars_order: String,
	symbol_width: u32,
	symbol_height: u32,
	// Textures for each symbol
	textures: HashMap<char, Box<[bool]>>
}

impl Font {
	pub fn new<F, IY, IX, O>(look: F, order: O, symbol_width: u32, symbol_height: u32) -> Result<Font, String>
		where F: IntoIterator<Item = (char, IY)>,
			  IY: IntoIterator<Item = IX>,
			  IX: IntoIterator<Item = bool>,
			  O: Into<String>
	{
		let mut map: HashMap<char, Box<[bool]>> = HashMap::new();
		let chars_order = order.into();

		for (symbol, look) in look {
			if chars_order.rfind(symbol).is_none() {
				return Err("'look' must contain textures for every symbol in 'order'".to_string());
			}

			let look: Vec<bool> = look.into_iter().fold(vec![], |mut prev, x| {
				prev.extend(x.into_iter());
				prev
			});
			map.insert(symbol, look.into_boxed_slice());
		}

		for symbol in chars_order.chars() {
			if map.get(&symbol).is_none() {
				return Err("'order' must contain every symbol in 'look'".to_string());
			}
		}

		Ok(
			Font {
				chars_order,
				symbol_width,
				symbol_height,
				textures: map,
			}
		)
	}

	pub fn all_symbols(&self) -> &String {
		&self.chars_order
	}

	pub fn symbol_size(&self) -> (u32, u32) {
		(self.symbol_width, self.symbol_height)
	}

	pub fn symbol_id(&self, symbol: char) -> Option<usize> {
		self.chars_order.rfind(symbol)
	}

	pub fn symbol_texture(&self, symbol: char) -> Option<&Box<[bool]>> {
		self.textures.get(&symbol)
	}

	pub fn make_scheme(&self) -> Result<Scheme, String> {
		let mut combiner = Combiner::pos_manual();

		let word_size = (self.all_symbols().len() as f64).log2().ceil() as u32;

		let mut sel = binary_selector_compact(word_size);
		sel.rotate(Rot::new(1, 0, 3));
		let sel_x_size = *sel.bounds().x();

		combiner.add("sel", sel).unwrap();
		let selector_pos = (self.chars_order.chars().count() as u32) / (self.symbol_width * self.symbol_height) + sel_x_size;
		combiner.pos().place("sel", (-(selector_pos as i32), 0, 0));
		combiner.pass_input("_", "sel/_", Some("binary")).unwrap();

		combiner.rect_vert("display", OR, self.symbol_width, self.symbol_height).unwrap();
		combiner.pos().place("display", (1, 0, 0));
		combiner.pass_output("_", "display", Some("graphics")).unwrap();

		for (i, symbol) in self.chars_order.chars().enumerate() {
			let i = i as u32;
			let y = i % self.symbol_width;
			let z = (i / self.symbol_width) % self.symbol_height;
			let neg_x = i / (self.symbol_width * self.symbol_height);

			let gate_name = format!("{}", i);

			let mut any_connections = false;

			for tex_x in 0..self.symbol_width {
				for tex_y in 0..self.symbol_height {
					let pixel_id = tex_y * self.symbol_width + tex_x;
					if self.symbol_texture(symbol).unwrap()[pixel_id as usize] {
						any_connections = true;

						combiner.connect(&gate_name, format!("display/{}_{}", tex_x, self.symbol_height - tex_y - 1));
					}
				}
			}

			if any_connections {
				combiner.add(&gate_name, AND).unwrap();
				combiner.connect(format!("sel/{}", i), &gate_name);
			} else {
				combiner.add(&gate_name, BlockBody::new(BlockType::Glass, (1, 1, 1))).unwrap();
			}
			combiner.pos().place_last((-(neg_x as i32), y as i32, z as i32));
		}

		match combiner.compile() {
			Ok((scheme, _)) => Ok(scheme),
			Err(error) => match error {
				CompileError::PositionerError(error) => panic!("Font is not created: {:?}", error),
				CompileError::ConnectionsOverflow { .. } => Err("Failed to create Font Scheme due to \
				connections overflow. Fonts with more than 255 symbols are not fully supported.".to_string())
			}
		}
	}

	pub fn make_sign_symb(&self, symbol: char, add_paddings: bool, fill_with: Scheme, bg_with: Scheme) -> Result<Scheme, String> {
		if fill_with.bounds() != bg_with.bounds() {
			return Err(format!("'fill_with' and 'bg_with' bounds must be equal ({:?} != {:?})", fill_with.bounds().tuple(), bg_with.bounds().tuple()));
		}

		let texture = match self.symbol_texture(symbol) {
			None => return Err(format!("Symbol '{}' was not found", symbol)),
			Some(texture) => texture,
		};
		let pixel = |x: u32, y: u32| if x < self.symbol_width && y < self.symbol_height {
			texture[(y * self.symbol_width + x) as usize]
		} else {
			false
		};

		let mut combiner = Combiner::pos_manual();

		let x_step = *fill_with.bounds().x();
		let y_step = *fill_with.bounds().y();

		let (w, h) = if add_paddings {
			(self.symbol_width + 1, self.symbol_height + 1)
		} else {
			(self.symbol_width, self.symbol_height)
		};
		for x in 0..w {
			for y in 0..h {
				let add_scheme = if pixel(x, y) { fill_with.clone() } else { bg_with.clone() };
				let name = format!("{}_{}", x, y);
				combiner.add(&name, add_scheme).unwrap();
				combiner.set_forcibly_used(name).unwrap();
				combiner.pos().place_last(( - ((x * x_step) as i32), (y * y_step) as i32, 0));
			}
		}

		let (mut scheme, _) = combiner.compile().unwrap();
		scheme.rotate(Rot::new(0, 0, 1));
		Ok(scheme)
	}

	pub fn make_sign_symb_def(&self, symbol: char, add_paddings: bool) -> Result<Scheme, String> {
		let mut fill_with: Scheme = BlockBody::new(BlockType::Plastic, (1, 1, 1)).into();
		let mut bg_with: Scheme = BlockBody::new(BlockType::Plastic, (1, 1, 1)).into();
		fill_with.full_paint("eeeeee");
		bg_with.full_paint("222222");
		self.make_sign_symb(symbol, add_paddings, fill_with, bg_with)
	}

	pub fn make_sign(&self, text: &str, fill_with: Scheme, bg_with: Scheme) -> Result<Scheme, String> {
		let mut cur_x = 0_i32;
		let mut cur_y = 0_i32;
		let mut next_y = 0_i32;

		let mut combiner = Combiner::pos_manual();

		for (i, symbol) in text.chars().enumerate() {
			if symbol == '\n' && self.symbol_texture('\n').is_none() {
				cur_x = 0;
				cur_y = next_y;
				continue;
			}

			let sign = self.make_sign_symb(symbol, true, fill_with.clone(), bg_with.clone())?;
			let (size_x, size_y, _) = sign.bounds().tuple();

			combiner.add(format!("{}", i), sign).unwrap();
			combiner.pos().place_last((-cur_y, -cur_x, 0));

			cur_x += size_y as i32;
			next_y = next_y.max(cur_y + size_x as i32)
		}

		let (scheme, _) = combiner.compile().unwrap();
		Ok(scheme)
	}

	pub fn make_sign_def(&self, text: &str) -> Result<Scheme, String> {
		let mut fill_with: Scheme = BlockBody::new(BlockType::Plastic, (1, 1, 1)).into();
		let mut bg_with: Scheme = BlockBody::new(BlockType::Plastic, (1, 1, 1)).into();
		fill_with.full_paint("eeeeee");
		bg_with.full_paint("222222");
		self.make_sign(text, fill_with, bg_with)
	}
}

pub fn main_font() -> Font {
	Font::new(MAIN_FONT, MAIN_FONT_SYMBOLS, 5, 9).unwrap()
}

pub fn numbers_font() -> Font {
	Font::new(NUMBERS, NUMBERS_SYMBOLS, 3, 5).unwrap()
}

pub fn hex_font() -> Font {
	Font::new(HEX, HEX_SYMBOLS, 3, 5).unwrap()
}