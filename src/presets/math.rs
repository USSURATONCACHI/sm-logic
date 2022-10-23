use crate::bind::Bind;
use crate::combiner::Combiner;
use crate::connection::ConnMap;
use crate::positioner::ManualPos;
use crate::presets::shapes_cube;
use crate::scheme::Scheme;
use crate::shape::vanilla::BlockType;
use crate::shape::vanilla::GateMode::{AND, NOR, OR, XOR};
use crate::util::{Facing, Point};

/// ***Inputs***: a, b.
///
/// ***Outputs***: _ (result).

///
/// Multiplies two numbers.
///
/// Send two numbers to 'a' and 'b' and a little while later their
/// product will be available on the default output.
///
/// Does not support threaded computations.
///
/// Have time complexity of `O(log(n))`, where `n` is word size -
/// `bits_before_point + bits_after_point`.
/// And logarithmic computations is really fast.
///
/// BUT it is really big and uses a lot of logic gates. Gates amount
/// complexity is `O(n * n * log(n))`.
///
/// Examples:
///
/// `big_multiplier(8, 0)` uses 171 logic gates after `.remove_unused()` (word size is 8).
/// `big_multiplier(32, 32)` uses 18213 logic gates after `.remove_unused()` (word size is 64).
///
/// It is good for small numbers but not for big numbers.
pub fn big_multiplier(bits_before_point: u32, bits_after_point: u32) -> Scheme {
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

/// ***Inputs***: _ (number).
///
/// ***Outputs***: _ (inverted number).

///
/// Inverts a binary number.
/// Has O(word_size) time complexity.
pub fn inverter(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add_iter([
		("const_signal", 	NOR),
		("const_signal_start", 	AND),
	]).unwrap();

	combiner.pos().place_iter([
		("const_signal", 	(2, -1, 0)),
		("const_signal_start", 	(1, -1, 0)),
	]);
	combiner.connect("const_signal_start", "const_signal");
	// Connect to first bits of carry and out
	combiner.connect_iter(["const_signal"], ["carry", "out"]);

	combiner.add_shapes_cube("or", (word_size, 1, 1), OR, Facing::PosY.to_rot()).unwrap();
	combiner.add_shapes_cube("nor", (word_size, 1, 1), NOR, Facing::PosZ.to_rot()).unwrap();
	combiner.add_shapes_cube("carry", (word_size, 1, 1), AND, Facing::PosZ.to_rot()).unwrap();
	combiner.add_shapes_cube("out", (word_size, 1, 1), XOR, Facing::NegY.to_rot()).unwrap();

	combiner.connect("or", "nor");
	combiner.connect("nor", "out");
	combiner.connect("nor", "carry");

	// Connection, that is shifted by +1X
	let shift_1 = ConnMap::new(
		|(point, _in_bounds), _out_bounds| Some(point + Point::new_ng(1, 0, 0))
	);
	combiner.custom("carry", "carry", shift_1.clone());
	combiner.custom("carry", "out", shift_1);

	combiner.pos().place_iter([
		("or", 		(0, 0, 0)),
		("nor", 	(1, 0, 0)),
		("carry", 	(2, 0, 0)),
		("out", 	(3, 0, 0)),
	]);
	combiner.pos().rotate_iter([
		("or", 		(0, 0, 1)),
		("nor", 	(0, 0, 1)),
		("carry", 	(0, 0, 1)),
		("out", 	(0, 0, 1)),
	]);

	let mut bind = Bind::new("_", "binary", (word_size, 1, 1));
	bind.connect_full("or");
	bind.gen_point_sectors("bit", |x, _, _| x.to_string()).unwrap();
	combiner.bind_input(bind).unwrap();

	let mut bind = Bind::new("_", "binary", (word_size, 1, 1));
	bind.connect_full("out");
	bind.gen_point_sectors("bit", |x, _, _| format!("{}", x)).unwrap();
	combiner.bind_output(bind).unwrap();

	let (scheme, _) = combiner.compile().unwrap();
	scheme
}


/// ***Inputs***: a, b, carry.
///
/// ***Outputs***: _ (result), carry.

///
/// Adds two numbers.
///
/// Send two binary numbers to 'a' and 'b', then `2 * word_size` ticks
/// later result of addition will be available on default output.
///
/// With some input time shifting it is possible to use this for 3-tick
/// threaded calculations. If you send each input bit with 2-tick delay
/// from previous bit, then there will be correct output with the same
/// delay between bits. Inputs can be 1-tick.
///
/// And the point of this is in that threaded case you can send
/// different pairs of numbers each two ticks and get correct result.
/// To remove delay between output bits just add reverse delay.
/// Threaded computations allow to add two numbers each two ticks
/// (20 times per second) no matter `word_size`. The only downside is
/// little delays in start and in the end during to bits delays.
pub fn adder(word_size: u32) -> Scheme {
	let mut adder = Combiner::pos_manual();

	adder.add("adder", adder_compact(word_size)).unwrap();
	adder.add_shapes_cube("a", (word_size, 1, 1), OR, Facing::PosY.to_rot()).unwrap();
	adder.add_shapes_cube("b", (word_size, 1, 1), OR, Facing::PosY.to_rot()).unwrap();

	adder.connect("a", "adder/a");
	adder.connect("b", "adder/b");

	adder.pass_output("_", "adder", None as Option<String>).unwrap();

	let mut inp_a = Bind::new("a", "binary", (word_size, 1u32, 1u32));
	inp_a.connect_full("a");
	inp_a.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	adder.bind_input(inp_a).unwrap();

	let mut inp_b = Bind::new("b", "binary", (word_size, 1u32, 1u32));
	inp_b.connect_full("b");
	inp_b.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	adder.bind_input(inp_b).unwrap();

	adder.pass_input("carry", "adder/carry", None as Option<String>).unwrap();
	adder.pass_output("carry", "adder/carry", None as Option<String>).unwrap();

	adder.pos().place_iter([
		("adder", (1, 0, 0)),
		("a", (0, 0, 0)),
		("b", (0, 0, 1)),
	]);

	adder.pos().rotate_iter([
		("a", (0, 0, 1)),
		("b", (0, 0, 1)),
	]);

	let (scheme, _invalid) = adder.compile().unwrap();
	scheme
}

/// ***Inputs***: a, b, carry.
///
/// ***Outputs***: _ (result), carry.

///
/// Adder without input protection. Inputs 'a' and 'b' should only be
/// connected into from one and only one gate for each bit, since AND
/// gates are used for calculations.
///
/// Send two binary numbers to 'a' and 'b', then `2 * word_size` ticks
/// later result of addition will be available on default output.
///
/// With some input time shifting it is possible to use this for 2-tick
/// threaded calculations. If you send each input bit with 2-tick delay
/// from previous bit, then there will be correct output with the same
/// delay between bits. Inputs can be 1-tick.
///
/// And the point of this is in that threaded case you can send
/// different pairs of numbers each two ticks and get correct result.
/// To remove delay between output bits just add reverse delay.
/// Threaded computations allow to add two numbers each two ticks
/// (20 times per second) no matter `word_size`. The only downside is
/// little delays in start and in the end during to bits delays.
pub fn adder_compact(word_size: u32) -> Scheme {
	let mut s = Combiner::pos_manual();

	s.add_shapes_cube("carry", (word_size, 1, 1), OR, (0, 0, 0)).unwrap();
	let and_line = shapes_cube((word_size, 1, 1), AND, (0, 0, 0));
	s.add_mul(["and_1", "and_2", "and_3"], and_line).unwrap();
	s.add_shapes_cube("res", (word_size, 1, 1), XOR, Facing::NegY.to_rot()).unwrap();

	s.pos().place_iter([
		("carry", (1, 0, 1)),
		("and_1", (0, 0, 0)),
		("and_2", (0, 0, 1)),
		("and_3", (1, 0, 0)),
		("res",   (2, 0, 0)),
	]);

	s.pos().rotate_iter([
		("carry", 	(0, 0, 1)),
		("and_1", 	(0, 0, 1)),
		("and_2", 	(0, 0, 1)),
		("and_3", 	(0, 0, 1)),
		("res", 	(0, 0, 1)),
	]);

	s.connect_iter(["carry"], ["res", "and_3", "and_1"]);

	// Connection, that is shifted by +1X
	let shift_1 = ConnMap::new(
		|(point, _in_bounds), _out_bounds| Some(point + Point::new_ng(1, 0, 0))
	);
	s.custom_iter(["and_1", "and_2", "and_3"], ["carry"], shift_1);

	let mut inp_a = Bind::new("a", "binary", (word_size, 1, 1));
	inp_a.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	inp_a.connect_full("and_1");
	inp_a.connect_full("and_2");
	inp_a.connect_full("res");
	s.bind_input(inp_a).unwrap();

	let mut inp_b = Bind::new("b", "binary", (word_size, 1, 1));
	inp_b.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	inp_b.connect_full("and_2");
	inp_b.connect_full("and_3");
	inp_b.connect_full("res");
	s.bind_input(inp_b).unwrap();

	let mut out = Bind::new("_", "binary", (word_size, 1, 1));
	out.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	out.connect_full("res");
	s.bind_output(out).unwrap();


	let mut carry_in = Bind::new("carry", "bit", (1, 1, 1));
	carry_in.connect_full("carry");
	s.bind_input(carry_in).unwrap();

	let mut carry_out = Bind::new("carry", "bit", (1, 1, 1));
	carry_out.connect_full(format!("and_1/_/{}_0_0", word_size as i32 - 1))
			.connect_full(format!("and_2/_/{}_0_0", word_size as i32 - 1))
			.connect_full(format!("and_3/_/{}_0_0", word_size as i32 - 1));
	s.bind_output(carry_out).unwrap();

	let (scheme, _invalid) = s.compile().unwrap();
	scheme
}
