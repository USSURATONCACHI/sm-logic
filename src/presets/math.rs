use crate::bind::Bind;
use crate::combiner::Combiner;
use crate::connection::ConnMap;
use crate::positioner::ManualPos;
use crate::presets::shapes_cube;
use crate::scheme::Scheme;
use crate::shape::vanilla::BlockType;
use crate::shape::vanilla::GateMode::{AND, NOR, OR, XOR};
use crate::util::{Facing, Point};

pub fn multiplier(bits_before_point: u32, bits_after_point: u32) -> Scheme {
	let size = bits_before_point + bits_after_point;

	let mut combiner = Combiner::pos_manual();

	let slots_kind = format!("binary[{}.{}]", bits_before_point, bits_after_point);

	combiner.add_shapes_cube("a", (size, 1, 1), OR, Facing::PosY.to_rot()).unwrap();
	combiner.pos().place_last((-2, 0, 0));
	combiner.pos().rotate_last((0, 0, 1));
	combiner.pass_input("a", "a", Some(slots_kind.clone())).unwrap();

	combiner.add_shapes_cube("b", (size, 1, 1), OR, Facing::PosY.to_rot()).unwrap();
	combiner.pos().place_last((-2, 0, 1));
	combiner.pos().rotate_last((0, 0, 1));
	combiner.pass_input("b", "b", Some(slots_kind.clone())).unwrap();

	// Actually, regular multiplier is pretty easy to create, BUT i
	// added a lot of gates usage optimizations here, and so the
	// function is really big

	// This multiplier uses about O(n * n * log(n)) gates, where n is
	// word size. And because it is so big, i added A LOT of gate usage
	// optimizations to this function.

	// start bit, end bit, path to each bit
	let mut prev_step: Vec<(u32, Vec<String>)> = vec![];

	// at first, multiply A by each digit of B separately
	for i in 0..size {
		// CREATE ROW SCHEME
		let line_start = (i as i32) - (bits_after_point as i32);
		let line_end = (line_start + (size as i32)).min(size as i32);

		let mut line = shapes_cube((size, 1, 1), AND, Facing::PosY.to_rot());
		let start_point = line.calculate_bounds().0.x().clone();	// calculate bounds -> start point -> x
		line.filter_shapes(|pos, _, _| *pos.x() >= line_start && *pos.x() < line_end);
		let offset = line.calculate_bounds().0.x().clone() - start_point;

		// ADD ROW TO TABLE
		let name = format!("table_{}", i);
		let start = offset;
		let end = offset + (*line.bounds().x() as i32);
		let bits = (start..end).map(|bit_id| format!("{}/_/{}_0_0", name, bit_id)).collect();
		prev_step.push((start as u32, bits));

		// ADD ROW TO SCHEME
		combiner.add(&name, line).unwrap();
		combiner.pos().rotate_last((0, 0, 1));
		combiner.pos().place_last((-1, offset, i as i32));

		let conn_offset = (i as i32) - (bits_after_point as i32);
		combiner.custom("a", &name, ConnMap::new(
			move |(point, _), _| Some(point + Point::new_ng(conn_offset, 0, 0))
		));

		combiner.dim(format!("b/_/{}_0_0", i), &name, (true, true, true));

	}

	// Then, add up all results
	let mut iteration = 0;
	while prev_step.len() > 1 {
		prev_step = add_rows_once(iteration, &mut combiner, prev_step);
		iteration += 1;
	}

	// bind output
	let mut bind = Bind::new("_", slots_kind, (size, 1, 1));

	if prev_step.len() == 1 {
		let (start, bits) = prev_step.into_iter().next().unwrap();

		for (bit_id, bit) in bits.into_iter().enumerate() {
			let bit_id = bit_id as i32 + start as i32;
			bind.connect(((bit_id as i32, 0, 0), (1, 1, 1)), bit);
		}
	}
	bind.gen_point_sectors("bit", |x, _, _| format!("{}", x)).unwrap();
	combiner.bind_output(bind).unwrap();

	let (mut scheme, _invalid) = combiner.compile().unwrap();
	scheme.replace_unused_with(BlockType::Glass);
	return scheme;
}

fn add_rows_once(iteration: i32, combiner: &mut Combiner<ManualPos>, rows_map: Vec<(u32, Vec<String>)>) -> Vec<(u32, Vec<String>)> {
	const ADDER_X_SIZE: i32 = 3;
	const ADDER_Z_SIZE: i32 = 2;

	let mut iter = rows_map.into_iter();
	let mut new_step: Vec<(u32, Vec<String>)> = vec![];
	let mut adder_id = 0;
	'main: loop {
		let (a_start, a_bits) = match iter.next() {
			None => break 'main,
			Some(word) => word,
		};

		match iter.next() {
			Some((b_start, b_bits)) => {
				// ------###### A; - is empty
				// ----#####	B
				// ----||+++>>> A + B; | is pass through, + is add, > is maybe add
				// add A and B up
				let new_row_start = a_start.min(b_start);
				let new_row_end = (a_start + a_bits.len() as u32).max(b_start + b_bits.len() as u32);
				let adder_start = a_start.max(b_start);
				let adder_end = (a_start + a_bits.len() as u32).min(b_start + b_bits.len() as u32);

				let adder_size = (adder_end - adder_start) as u32;
				let adder = adder_compact(adder_size);
				let adder_name = format!("adder_{}_{}", iteration, adder_id);

				combiner.add(&adder_name, adder).unwrap();
				combiner.pos().place_last((iteration * ADDER_X_SIZE, adder_start as i32, adder_id * ADDER_Z_SIZE));

				let bit_adder_size = new_row_end - adder_end;
				let bit_adder_name = format!("zero_one_{}_{}", iteration, adder_id);
				if bit_adder_size > 0 {
					let bit_adder = _add_0_or_1(bit_adder_size);

					combiner.add(&bit_adder_name, bit_adder).unwrap();
					combiner.pos().place_last((iteration * ADDER_X_SIZE, adder_end as i32, adder_id * ADDER_Z_SIZE));
					combiner.connect(format!("{}/carry_bit", adder_name), format!("{}/bit", bit_adder_name));
				}

				let new_row_size = (new_row_end - new_row_start) as usize;
				let mut new_row: Vec<String> = Vec::with_capacity(new_row_size);

				// Add bits that does not need to be added
				for bit in new_row_start..adder_start {
					if bit < a_start {
						// add bit from B
						new_row.push(b_bits[(bit - b_start) as usize].clone());
					} else {
						// add bit from A
						new_row.push(a_bits[(bit - a_start) as usize].clone());
					}
				}

				// Add all the bits that need to be added
				for bit in adder_start..adder_end {
					let a_bit_id = (bit - a_start) as usize;
					if a_bit_id < a_bits.len() {
						let a_bit_path = a_bits[a_bit_id].clone();
						combiner.connect(a_bit_path, format!("{}/a/{}", adder_name, bit - adder_start));
					}

					let b_bit_id = (bit - b_start) as usize;
					if b_bit_id < b_bits.len() {
						let b_bit_path = b_bits[b_bit_id].clone();
						combiner.connect(b_bit_path, format!("{}/b/{}", adder_name, bit - adder_start));
					}

					new_row.push(format!("{}/_/{}", adder_name, bit - adder_start));
				}

				// Add bits that does not need to be added
				for bit in adder_end..new_row_end {
					let a_bit_id = (bit - a_start) as usize;
					if a_bit_id < a_bits.len() {
						let a_bit_path = a_bits[a_bit_id].clone();
						combiner.connect(a_bit_path, format!("{}/data/{}", bit_adder_name, bit - adder_end));
					}

					let b_bit_id = (bit - b_start) as usize;
					if b_bit_id < b_bits.len() {
						let b_bit_path = b_bits[b_bit_id].clone();
						combiner.connect(b_bit_path, format!("{}/data/{}", bit_adder_name, bit - adder_end));
					}

					new_row.push(format!("{}/_/{}", bit_adder_name, bit - adder_end));
				}

				new_step.push((new_row_start, new_row));
				adder_id += 1;
			}

			None => {
				// just pass A through
				new_step.push((a_start, a_bits));
			}
		}
	}

	new_step
}

fn _add_0_or_1(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add("0_or_1", OR).unwrap();
	combiner.pos().place_last((0, 0, 0));
	combiner.pos().rotate_last(Facing::NegX.to_rot());
	combiner.connect("0_or_1", "0_and");
	combiner.connect("0_or_1", "0_xor");

	let mut bind = Bind::new("bit", "binary", (1, 1, 1));
	bind.connect_full("0_or_1");
	combiner.bind_input(bind).unwrap();

	for i in 0..word_size {
		combiner.add(format!("{}_and", i), AND).unwrap();
		combiner.pos().place_last((1, i as i32, 0));

		combiner.add(format!("{}_xor", i), XOR).unwrap();
		combiner.pos().place_last((2, i as i32, 0));
		combiner.pos().rotate_last(Facing::PosX.to_rot());

		combiner.connect(format!("{}_and", i), format!("{}_and", i + 1));
		combiner.connect(format!("{}_and", i), format!("{}_xor", i + 1));
	}

	let mut input = Bind::new("data", "binary", (word_size, 1, 1));
	input.connect_func(|x, _, _| Some(format!("{}_and", x)));
	input.connect_func(|x, _, _| Some(format!("{}_xor", x)));
	input.gen_point_sectors("bit", |x, _, _| format!("{}", x)).unwrap();
	combiner.bind_input(input).unwrap();

	let mut output = Bind::new("_", "binary", (word_size, 1, 1));
	output.connect_func(|x, _, _| Some(format!("{}_xor", x)));
	output.gen_point_sectors("bit", |x, _, _| format!("{}", x)).unwrap();
	combiner.bind_output(output).unwrap();

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}


pub fn inverter(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add_iter([
		("-1_next", 	NOR),
		("no_inputs", 	AND),
	]).unwrap();

	combiner.pos().place_iter([
		("-1_next", 	(2, -1, 0)),
		("no_inputs", 	(1, -1, 0)),
	]);

	combiner.connect("no_inputs", "-1_next");

	for i in 0..word_size {
		combiner.add_iter([
			(format!("{}_inp", i), 	OR),
			(format!("{}_nor", i), 	NOR),
			(format!("{}_next", i), AND),
			(format!("{}_out", i), 	XOR),
		]).unwrap();

		combiner.connect(format!("{}_inp", i), format!("{}_nor", i));
		combiner.connect(format!("{}_nor", i), format!("{}_out", i));
		combiner.connect(format!("{}_nor", i), format!("{}_next", i));

		combiner.connect(format!("{}_next", (i as i32) - 1), format!("{}_next", i));
		combiner.connect(format!("{}_next", (i as i32) - 1), format!("{}_out", i));

		combiner.pos().place_iter([
			(format!("{}_inp", i), 	(0, i as i32, 0)),
			(format!("{}_nor", i), 	(1, i as i32, 0)),
			(format!("{}_next", i), (2, i as i32, 0)),
			(format!("{}_out", i), 	(3, i as i32, 0)),
		]);

		combiner.pos().rotate(format!("{}_inp", i), Facing::NegX.to_rot());
		combiner.pos().rotate(format!("{}_out", i), Facing::PosX.to_rot());
	}

	let mut bind = Bind::new("_", "binary", (word_size, 1, 1));
	bind.connect_func(|x, _, _| Some(format!("{}_inp", x)));
	bind.gen_point_sectors("bit", |x, _, _| format!("{}", x)).unwrap();
	combiner.bind_input(bind).unwrap();

	let mut bind = Bind::new("_", "binary", (word_size, 1, 1));
	bind.connect_func(|x, _, _| Some(format!("{}_out", x)));
	bind.gen_point_sectors("bit", |x, _, _| format!("{}", x)).unwrap();
	combiner.bind_output(bind).unwrap();

	let (scheme, _) = combiner.compile().unwrap();
	scheme
}



pub fn adder(word_size: u32) -> Scheme {
	_adder(word_size, adder_section())
}

pub fn adder_compact(word_size: u32) -> Scheme {
	_adder(word_size, adder_section_compact())
}

fn _adder(word_size: u32, section: Scheme) -> Scheme {
	let mut adder = Combiner::pos_manual();

	for i in 0..word_size {
		adder.add(format!("{}", i), section.clone()).unwrap();
		adder.pos().place_last((0, i as i32, 0));

		adder.connect(format!("{}", i), format!("{}", i+1));
	}

	let mut bind = Bind::new("a", "binary", (word_size, 1u32, 1u32));
	for x in 0..(word_size as i32) {
		bind.connect(((x, 0, 0), (1, 1, 1)), format!("{}/a", x));
		bind.add_sector(format!("{}", x), (x, 0, 0), (1, 1, 1), "binary").unwrap();
	}
	adder.bind_input(bind).unwrap();

	let mut bind = Bind::new("b", "binary", (word_size, 1u32, 1u32));
	for x in 0..(word_size as i32) {
		bind.connect(((x, 0, 0), (1, 1, 1)), format!("{}/b", x));
		bind.add_sector(format!("{}", x), (x, 0, 0), (1, 1, 1), "binary").unwrap();
	}
	adder.bind_input(bind).unwrap();

	let mut bind = Bind::new("_", "binary", (word_size, 1u32, 1u32));
	bind.connect_func(|x, _, _| Some(format!("{}/res", x)));
	bind.gen_point_sectors("bit", |x, _, _| format!("{}", x)).unwrap();
	adder.bind_output(bind).unwrap();

	let mut bind = Bind::new("carry_bit", "bit", (1, 1, 1));
	bind.connect_full(format!("{}", word_size as i32 - 1));
	adder.bind_output(bind).unwrap();

	let (scheme, _) = adder.compile().unwrap();
	scheme
}

fn adder_section() -> Scheme {
	let mut s = Combiner::pos_manual();

	s.add_mul(["a", "b", "_"], OR).unwrap();
	s.add_mul(["and_1", "and_2", "and_3"], AND).unwrap();
	s.add("res", XOR).unwrap();

	s.pos().place_iter([
		("a", (0, 0, 0)),
		("b", (0, 0, 1)),
		("_", (2, 0, 1)),
		("and_1", (1, 0, 0)),
		("and_2", (1, 0, 1)),
		("and_3", (2, 0, 0)),
		("res", (3, 0, 0)),
	]);

	s.pos().rotate("a", Facing::NegX.to_rot());
	s.pos().rotate("b", Facing::NegX.to_rot());
	s.pos().rotate("res", Facing::PosX.to_rot());

	let mut bind = Bind::new("a", "bit", (1, 1, 1));
	bind.connect_full("a");
	s.bind_input(bind).unwrap();

	let mut bind = Bind::new("b", "bit", (1, 1, 1));
	bind.connect_full("b");
	s.bind_input(bind).unwrap();

	let mut bind = Bind::new("_", "bit", (1, 1, 1));
	bind.connect_full("_");
	s.bind_input(bind).unwrap();

	s.connect_iter(["a", "b", "_"], ["res"]);

	s.connect_iter(["a"], ["and_1", "and_2"]);
	s.connect_iter(["b"], ["and_2", "and_3"]);
	s.connect_iter(["_"], ["and_3", "and_1"]);

	let mut bind = Bind::new("_", "bit", (1, 1, 1));
	bind.connect_full("and_1")
		.connect_full("and_2")
		.connect_full("and_3");
	s.bind_output(bind).unwrap();

	let mut bind = Bind::new("res", "bit", (1, 1, 1));
	bind.connect_full("res");
	s.bind_output(bind).unwrap();

	let (scheme, _) = s.compile().unwrap();
	scheme
}

fn adder_section_compact() -> Scheme {
	let mut s = Combiner::pos_manual();

	s.add_mul(["_"], OR).unwrap();
	s.add_mul(["and_1", "and_2", "and_3"], AND).unwrap();
	s.add("res", XOR).unwrap();

	s.pos().place_iter([
		("_", (1, 0, 1)),
		("and_1", (0, 0, 0)),
		("and_2", (0, 0, 1)),
		("and_3", (1, 0, 0)),
		("res", (2, 0, 0)),
	]);

	s.pos().rotate("res", Facing::PosX.to_rot());

	let mut bind = Bind::new("a", "bit", (1, 1, 1));
	bind.connect_full("res");
	bind.connect_full("and_1");
	bind.connect_full("and_2");
	s.bind_input(bind).unwrap();

	let mut bind = Bind::new("b", "bit", (1, 1, 1));
	bind.connect_full("res");
	bind.connect_full("and_2");
	bind.connect_full("and_3");
	s.bind_input(bind).unwrap();

	let mut bind = Bind::new("_", "bit", (1, 1, 1));
	bind.connect_full("_");
	s.bind_input(bind).unwrap();

	s.connect_iter(["_"], ["res", "and_3", "and_1"]);

	let mut bind = Bind::new("_", "bit", (1, 1, 1));
	bind.connect_full("and_1")
		.connect_full("and_2")
		.connect_full("and_3");
	s.bind_output(bind).unwrap();

	let mut bind = Bind::new("res", "bit", (1, 1, 1));
	bind.connect_full("res");
	s.bind_output(bind).unwrap();

	let (scheme, _) = s.compile().unwrap();
	scheme
}