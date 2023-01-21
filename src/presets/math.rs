use crate::bind::Bind;
use crate::combiner::Combiner;
use crate::connection::{ConnMap};
use crate::positioner::ManualPos;
use crate::presets::{connect_safe, input_filter_rational, make_rational_bind, shapes_cube, shift_connection};
use crate::scheme::Scheme;
use crate::shape::vanilla::{BlockType, Timer};
use crate::shape::vanilla::GateMode::{AND, NOR, OR, XOR};
use crate::util::{Facing, MAX_CONNECTIONS, Point};

/// ***Inputs***: start,
/// a, a_rational,
/// b, b_rational.
///
/// ***Outputs***:
/// _ (result), rational,
/// same_size, same_size_rational.

///
/// Multiplies two numbers.
///
/// Send two binary numbers to 'a' and 'b' input and a 1-tick signal
/// to 'start' input simultaneously. Then, linear time later product
/// will be available on the output.
///
/// `word_size` is `bits_after_point + bits_after_point`.
///
/// Does not support threaded computations.
///
/// <br>
/// Time complexity: `O(word_size)`.
///
/// Space complexity: `O(word_size)`.
///
/// (`O(word_size)`, a bit more than `2 * word_size` ticks, to be more exact)
pub fn multiplier(bits_before_point: u32, bits_after_point: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	let word_size = bits_before_point + bits_after_point;

	combiner.add("a", input_filter_rational(bits_before_point, bits_after_point)).unwrap();
	combiner.pass_input("a", "a", None as Option<String>).unwrap();
	combiner.pass_input("a_rational", "a/rational", None as Option<String>).unwrap();

	combiner.add("b", input_filter_rational(bits_before_point, bits_after_point)).unwrap();
	combiner.pass_input("b", "b", None as Option<String>).unwrap();
	combiner.pass_input("b_rational", "b/rational", None as Option<String>).unwrap();

	// Each 3 ticks number shifts one bit down
	{
		let left_shift = bits_before_point as i32 - 1;
		combiner.add_shapes_cube("a_shifter_or", (word_size * 2 - 1, 1, 1), OR, Facing::NegY.to_rot()).unwrap();
		combiner.add_shapes_cube("a_shifter_timer", (word_size * 2 - 1, 1, 1), Timer::new(1), Facing::NegY.to_rot()).unwrap();

		combiner.connect("a_shifter_or", "a_shifter_timer");
		combiner.custom("a_shifter_timer", "a_shifter_or", shift_connection((-1, 0, 0)));
		combiner.custom("a", "a_shifter_or", shift_connection((left_shift + bits_after_point as i32, 0, 0)));

		combiner.add_shapes_cube("b_shifter_or", (word_size, 1, 1), OR, Facing::NegY.to_rot()).unwrap();
		combiner.add_shapes_cube("b_shifter_timer", (word_size, 1, 1), Timer::new(1), Facing::NegY.to_rot()).unwrap();
		combiner.connect("b_shifter_or", "b_shifter_timer");
		combiner.custom("b_shifter_timer", "b_shifter_or", shift_connection((1, 0, 0)));
		combiner.connect("b", "b_shifter_or");
	}

	combiner.add_shapes_cube("intersection", (word_size * 2 - 1, 1, 1), AND, Facing::NegY.to_rot()).unwrap();
	connect_safe(
		&mut combiner,
		(0..(2 * word_size - 1)).map(|i| format!("intersection/_/{}_0_0", i)),
		|combiner, i| {
			let name = format!("support_timer_{}", i);
			combiner.add(&name, Timer::new(1)).unwrap();
			combiner.pos().place_last((3, i as i32, 2));
			name
		},
		Some(format!("b_shifter_timer/_/{}_0_0", word_size as i32 - 1)),
		false
	).unwrap();

	combiner.connect("a_shifter_timer", "intersection");

	// Adder
	combiner.add("adder", adder_compact(word_size * 2)).unwrap();
	combiner.add_shapes_cube("adder_cycle_1", (word_size * 2, 1, 1), AND, (0, 0, 0)).unwrap();
	combiner.add_shapes_cube("adder_cycle_2", (word_size * 2, 1, 1), AND, (0, 0, 0)).unwrap();
	combiner.connect("adder", "adder_cycle_1");
	combiner.connect("adder_cycle_1", "adder_cycle_2");
	combiner.connect("adder_cycle_2", "adder/b");

	let resets = connect_safe(
		&mut combiner,
		(0..(2 * word_size)).map(|i| format!("adder_cycle_2/_/{}_0_0", i)),
		|combiner, i| {
			let name = format!("reset_nor_{}", i);
			combiner.add(&name, NOR).unwrap();
			combiner.pos().place_last((1, i as i32, 2));
			name
		},
		None,
		false
	).unwrap();

	for i in 0..resets {
		combiner.connect_iter(["start", "start_1", "start_2"], [format!("reset_nor_{}", i)]);
	}

	combiner.connect("intersection", "adder/a");

	// Outputs
	let mut output_def = Bind::new("_", "binary", (word_size * 2, 1, 1));
	output_def.connect_full("adder");
	output_def.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	combiner.bind_output(output_def).unwrap();

	let output_rational = make_rational_bind("rational", "adder", bits_before_point * 2, bits_after_point * 2, bits_after_point * 2, 0);
	combiner.bind_output(output_rational).unwrap();

	let mut same_size = Bind::new("same_size", "binary", (word_size, 1, 1));
	same_size.custom_full("adder", shift_connection(((bits_after_point as i32) * 2, 0, 0)));
	same_size.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	combiner.bind_output(same_size).unwrap();

	let same_size_rational = make_rational_bind(
		"same_size_rational", "adder", bits_before_point,
		bits_after_point, bits_after_point * 2, bits_after_point
	);
	combiner.bind_output(same_size_rational).unwrap();

	// Outputs end

	combiner.add_mul(["start", "start_1", "start_2"], OR).unwrap();
	combiner.connect("start", "start_1");
	combiner.connect("start_1", "start_2");
	combiner.connect_iter(["start", "start_1", "start_2"], ["adder/reset"]);
	combiner.dim_iter(["start", "start_1", "start_2"], ["a/activator", "b/activator"], (true, true, true));
	combiner.pass_input("start", "start", Some("logic")).unwrap();

	combiner.pos().place_iter([
		("a", (0, 0, 0)),
		("b", (0, 0, 1)),

		("a_shifter_or", (2, -(bits_after_point as i32), 0)),
		("a_shifter_timer", (4, -(bits_after_point as i32), 0)),
		("b_shifter_or", (2, 0, 1)),
		("b_shifter_timer", (4, 0, 1)),
		("intersection", (5, -(bits_after_point as i32), 0)),
		("adder", (6, -(bits_after_point as i32), 0)),
		("adder_cycle_1", (5, -(bits_after_point as i32), 2)),
		("adder_cycle_2", (5, -(bits_after_point as i32), 1)),

		("start", (0, 0, 0)),
		("start_1", (0, 0, 1)),
		("start_2", (0, 0, 2)),
	]);
	combiner.pos().rotate("start", Facing::NegX.to_rot());

	combiner.pos().rotate_iter(
		[
			"a_shifter_or", "a_shifter_timer",
			"b_shifter_or", "b_shifter_timer",
			"intersection", "adder_cycle_1",
			"adder_cycle_2"
		].into_iter()
			.map(|x| (x, (0, 0, 1)))
	);

	let (scheme, _) = combiner.compile().unwrap();
	scheme
}


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
///
/// ***Time complexity***: `O(word_size.log2())`.
///
/// ***Space complexity***: `O(word_size.pow(2) * word_size.log2())`.
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

/// Utility function for `big_multiplier`.
/// Adds adders to combiner to add each pair of rows.
///
/// Function is so large and ugly because it is a victim of a bunch of
/// shape usage optimizations. I was trying to make it use as few logic
/// gates as possible.
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

/// ***Inputs***: data, bit.
///
/// ***Outputs***: _ (number).

///
/// Allows to add single bit (1-digit binary number) to a binary number.
///
/// Is only needed as a part of `big_multiplier`.
///
/// ***Time complexity***: `O(word_size)` (exactly `word_size` ticks).
///
/// ***Space complexity***: `O(word_size)` (exactly `2 * word_size + 1` gates).
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
///
/// Theoretically allows for 1-tick threaded calculations (1-tick delay
/// between input numbers (like a number each tick), and 1-tick delay
/// between bits). But I have not checked yet.
///
/// ***Time complexity***: `O(word_size)` (exactly `word_size` ticks).
///
/// ***Space complexity***: `O(word_size)` (exactly `4 * word_size + 2` gates).
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
///
/// ***Time complexity***: `O(word_size)` (exactly `word_size * 2` ticks).
///
/// ***Space complexity***: `O(word_size)` (exactly `word_size * 7` gates).
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
///
/// ***Time complexity***: `O(word_size)` (exactly `word_size * 2` ticks).
///
/// ***Space complexity***: `O(word_size)` (exactly `word_size * 5` gates).
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

/// ***Inputs***: _ (data), reset.
///
/// ***Outputs***: _ (data).

///
/// Adds numbers on input to its buffer (output).
/// To set buffer value to zero, send 1-tick signal to 'reset' input.
///
/// Input numbers should be sent with step of 3 (or multiples of 3) ticks.
///
/// A simple add-on of 'adder', that connects adder output to it's
/// input, guarantees that timings are correct and smoothes output
/// signal.
///
/// You can do such a scheme manually by just connecting adder output
/// to one of it's inputs, but then you will have to make timings
/// control manually.
///
/// ***Time complexity***: `O(word_size)`? No real measurements were made for
/// this scheme, but theoretically it should be about
/// `2 * word_size + 6` ticks delay between input and output.
///
/// ***Space complexity***: `O(word_size)`. To be exact:
///
/// `16 * word_size + 11 + word_size / MAX_CONNECTIONS` shapes (gates
/// and timers).
pub fn adder_mem(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();
	combiner.set_debug_name("presets::math::adder_mem");

	// ADDER ITSELF
	combiner.add("adder", adder_compact(word_size)).unwrap();

	combiner.add_shapes_cube("adder_inp", (word_size, 1, 1), AND, (0, 0, 0)).unwrap();
	combiner.add_shapes_cube("adder_cycle_1", (word_size, 1, 1), AND, (0, 0, 0)).unwrap();
	combiner.add_shapes_cube("adder_cycle_2", (word_size, 1, 1), AND, (0, 0, 0)).unwrap();

	combiner.connect("adder", "adder_cycle_1");
	combiner.connect("adder_cycle_1", "adder_cycle_2");
	combiner.connect("adder_cycle_2", "adder/b");
	combiner.connect("adder_inp", "adder/a");

	combiner.pos().place_iter([
		("adder", 		  (2, 0, 0)),
		("adder_inp", 	  (1, 0, 0)),
		("adder_cycle_1", (4, 0, 1)),
		("adder_cycle_2", (1, 0, 1)),
	]);

	combiner.pos().rotate_iter([
		("adder_inp", (0, 0, 1)),
		("adder_cycle_1", (0, 0, 1)),
		("adder_cycle_2", (0, 0, 1)),
	]);

	// TICKGEN
	combiner.add_iter([
		("const_signal_0", AND),
		("const_signal_1", NOR),
		("1_tick_0", NOR),
		("1_tick_1", AND),
		("tickgen_0", OR),
		("tickgen_1", AND),
		("tickgen_2", AND),
	]).unwrap();
	combiner.connect("const_signal_0", "const_signal_1");
	combiner.connect_iter(["const_signal_1"], ["1_tick_0", "1_tick_1"]);
	combiner.connect("1_tick_0", "1_tick_1");
	combiner.connect("1_tick_1", "tickgen_0");
	combiner.connect("tickgen_0", "tickgen_1");
	combiner.connect("tickgen_1", "tickgen_2");
	combiner.connect("tickgen_2", "tickgen_0");

	combiner.pos().place_iter([
		("const_signal_0", (1, word_size as i32, 0)),
		("const_signal_1", (1, word_size as i32, 1)),
		("1_tick_0", (2, word_size as i32, 0)),
		("1_tick_1", (2, word_size as i32, 1)),
		("tickgen_0", (3, word_size as i32, 0)),
		("tickgen_1", (3, word_size as i32, 1)),
		("tickgen_2", (4, word_size as i32, 0)),
	]);

	combiner.pos().rotate_iter(
		[
			"const_signal_0", "const_signal_1", "1_tick_0", "1_tick_1",
			"tickgen_0", "tickgen_1", "tickgen_2"
		].into_iter().map(|name| (name, Facing::PosY.to_rot()))
	);

	// INPUT TIMINGS FILTER
	combiner.add_shapes_cube("inp_0", (word_size, 1, 1), OR, Facing::PosY.to_rot()).unwrap();
	combiner.add_shapes_cube("inp_1", (word_size, 1, 1), OR, Facing::PosY.to_rot()).unwrap();
	combiner.add_shapes_cube("inp_2", (word_size, 1, 1), OR, Facing::PosY.to_rot()).unwrap();
	combiner.add_shapes_cube("input_filter", (word_size, 1, 1), Timer::new(1), Facing::NegY.to_rot()).unwrap();

	combiner.connect("inp_0", "inp_1");
	combiner.connect("inp_0", "inp_2");
	combiner.connect("inp_1", "inp_2");

	combiner.connect("tickgen_0", "input_filter");
	combiner.connect_iter(["inp_2", "input_filter"], ["adder_inp"]);
	combiner.custom("input_filter", "input_filter", shift_connection((1, 0, 0)));

	combiner.pos().place_iter([
		("inp_0", (0, 0, 0)),
		("inp_1", (0, 0, 1)),
		("inp_2", (0, 0, 2)),
		("input_filter", (2, 0, 2)),
	]);

	combiner.pos().rotate_iter(
		[
			"inp_0", "inp_1", "inp_2", "input_filter",
			"out_buffer_0", "out_buffer_1", "out_buffer_2"
		].into_iter().map(|name| (name, (0, 0, 1)))
	);

	// INPUT ITSELF
	let mut input = Bind::new("_", "binary", (word_size, 1, 1));
	input.connect_full("inp_0").connect_full("inp_2");
	input.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	combiner.bind_input(input).unwrap();

	// OUTPUT TIMINGS FILTER
	combiner.add_shapes_cube("out_buffer_0", (word_size, 1, 1), OR, Facing::NegY.to_rot()).unwrap();
	combiner.add_shapes_cube("out_buffer_1", (word_size, 1, 1), OR, Facing::NegY.to_rot()).unwrap();
	combiner.add_shapes_cube("out_buffer_2", (word_size, 1, 1), OR, Facing::NegY.to_rot()).unwrap();
	combiner.connect("out_buffer_0", "out_buffer_1");
	combiner.connect("out_buffer_0", "out_buffer_2");
	combiner.connect("out_buffer_1", "out_buffer_2");
	combiner.pos().place("out_buffer_0", (6, 0, 0));
	combiner.pos().place("out_buffer_1", (6, 0, 1));
	combiner.pos().place("out_buffer_2", (6, 0, 2));

	for i in 0..word_size {
		let timer = format!("out_timer_{}", i);
		combiner.add(&timer, Timer::new( (word_size - i - 1) * 2 )).unwrap();
		combiner.pos().place_last((5, i as i32, 0));
		combiner.pos().rotate_last(Facing::PosZ.to_rot());

		combiner.connect(format!("adder/_/{}", i), &timer);
		combiner.connect(&timer, format!("out_buffer_0/_/{}_0_0", i));
		combiner.connect(&timer, format!("out_buffer_1/_/{}_0_0", i));
		combiner.connect(&timer, format!("out_buffer_2/_/{}_0_0", i));
	}

	// OUTPUT ITSELF
	let mut output = Bind::new("_", "binary", (word_size, 1, 1));
	output.connect_full("out_buffer_2");
	output.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	combiner.bind_output(output).unwrap();

	// INPUT TO RESET
	combiner.add_mul(["reset_0", "reset_1", "reset_2"], OR).unwrap();

	combiner.connect("reset_0", "reset_1");
	combiner.connect_iter(["reset_0", "reset_1"], ["reset_2"]);

	let mut reset_nor_name = "none".to_string();
	for conn_number in 0..word_size {
		let nor_gate_id = conn_number / MAX_CONNECTIONS;

		if conn_number % MAX_CONNECTIONS == 0 {
			reset_nor_name = format!("reset_nor_{}", nor_gate_id);
			combiner.add(&reset_nor_name, NOR).unwrap();
			combiner.pos().place_last((1 + nor_gate_id as i32, word_size as i32, 2));

			combiner.connect_iter(["reset_0", "reset_1", "reset_2"], [&reset_nor_name]);
		}
		combiner.connect(&reset_nor_name, format!("adder_cycle_1/_/{}_0_0", conn_number));
	}

	combiner.pos().place_iter([
		("reset_0", (0, word_size as i32, 0)),
		("reset_1", (0, word_size as i32, 1)),
		("reset_2", (0, word_size as i32, 2)),
	]);

	combiner.pos().rotate_iter([
		("reset_0", Facing::NegX.to_rot()),
		("reset_1", Facing::NegX.to_rot()),
		("reset_2", Facing::NegX.to_rot()),
	]);

	let mut reset = Bind::new("reset", "logic", (1, 1, 1));
	reset.connect_full("reset_0").connect_full("reset_2");
	combiner.bind_input(reset).unwrap();

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

/// ***Inputs***: a, b.
///
/// ***Outputs***: a>b, a=b, a<b.

///
/// Checks if one binary number is greater, equal or less than another.
///
/// Computes output in exactly 4 ticks no matter the size.
///
/// Size limit: word_size could be up to 255. If more - connections
/// overflow will happen.
///
/// Does not allow for threaded calculations.
///
/// ***Time complexity***: `O(1)` (exactly `4` ticks).
///
/// ***Space complexity***: `O(word_size)` (`word_size * 5 + 1` gates, if `word_size > 0`, to be exact)
pub fn fast_compare(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();
	combiner.set_debug_name("presets::math::comparator");

	combiner.add_shapes_cube("a", (word_size, 1, 1), OR, Facing::PosZ.to_rot()).unwrap();
	combiner.add_shapes_cube("and_a", (word_size, 1, 1), AND, Facing::PosZ.to_rot()).unwrap();
	//combiner.add_shapes_cube("diff_xor", (word_size, 1, 1), XOR, Facing::PosZ.to_rot()).unwrap();
	combiner.add_shapes_cube("and_b", (word_size, 1, 1), AND, Facing::PosZ.to_rot()).unwrap();
	combiner.add_shapes_cube("b", (word_size, 1, 1), OR, Facing::PosZ.to_rot()).unwrap();


	// diff_xor
	// This all is needed, so it won't break with word_size > 254
	for i in 0..word_size {
		let mut diff_xor_name = format!("diff_xor_{}/_/0_0_0", i);
		combiner.connect(&diff_xor_name, format!("and_a/_/{}_0_0", i));
		combiner.connect(&diff_xor_name, format!("and_b/_/{}_0_0", i));
		combiner.connect(format!("a/_/{}_0_0", i), &diff_xor_name);
		combiner.connect(format!("b/_/{}_0_0", i), &diff_xor_name);

		let mut xor_count = 1;
		let mut diff_xor_conns = 2;
		for check_nor in 0..i {
			if diff_xor_conns % (MAX_CONNECTIONS as i32) == 0 {
				diff_xor_name = format!("diff_xor_{}/_/{}_0_0", i, diff_xor_conns / (MAX_CONNECTIONS as i32));
				combiner.connect(format!("a/_/{}_0_0", i), &diff_xor_name);
				combiner.connect(format!("b/_/{}_0_0", i), &diff_xor_name);
				xor_count += 1;
			}

			let conn_to_check_nor_id = i - check_nor;
			let check_nor_gate_id = conn_to_check_nor_id / MAX_CONNECTIONS;

			combiner.connect(&diff_xor_name, format!("check_nor_{}/_/{}_0_0", check_nor, check_nor_gate_id));
			diff_xor_conns += 1;
		}


		combiner.add_shapes_cube(format!("diff_xor_{}", i), (xor_count, 1, 1), XOR, Facing::PosX.to_rot()).unwrap();
		combiner.pos().place_last((2, i as i32, 0));
		combiner.pos().rotate_last((0, -1, 0));

		combiner.connect_iter([format!("a/_/{}_0_0", i), format!("b/_/{}_0_0", i)], [format!("diff_xor_{}", i)]);

		// also add check_nor gates
		let check_nor_total_conns = word_size - i - 1;
		let check_nor_size = (check_nor_total_conns + MAX_CONNECTIONS - 1) / MAX_CONNECTIONS;
		let check_nor_name = format!("check_nor_{}", i);
		combiner.add_shapes_cube(&check_nor_name, (check_nor_size, 1, 1), NOR, Facing::PosX.to_rot()).unwrap();
		combiner.pos().place_last((2, i as i32, xor_count as i32));
		combiner.pos().rotate_last((0, -1, 0));

		combiner.dim(&check_nor_name, format!("and_a/_/{}_0_0", i), (true, true, true));
		combiner.dim(&check_nor_name, format!("and_b/_/{}_0_0", i), (true, true, true));
	}
	combiner.dim("check_nor_0", "a_eq_b", (true, true, true));

	combiner.connect("a", "and_a");
	combiner.connect("b", "and_b");

	combiner.add("a_is_bigger", OR).unwrap();
	combiner.add("b_is_bigger", OR).unwrap();
	combiner.add("a_eq_b", AND).unwrap();


	let mut input_a = Bind::new("a", "binary", (word_size, 1, 1));
	let mut input_b = Bind::new("b", "binary", (word_size, 1, 1));
	input_a.connect_full("a");
	input_b.connect_full("b");
	input_a.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	input_b.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	combiner.bind_input(input_a).unwrap();
	combiner.bind_input(input_b).unwrap();

	for i in 0..word_size {
		let a_name = format!("a_is_bigger_{}", i / MAX_CONNECTIONS);
		let b_name = format!("b_is_bigger_{}", i / MAX_CONNECTIONS);
		if i % MAX_CONNECTIONS == 0 {
			combiner.add(&a_name, OR).unwrap();
			combiner.add(&b_name, OR).unwrap();

			combiner.pos().place(&a_name, (1, 1, 1 + (i / MAX_CONNECTIONS) as i32));
			combiner.pos().place(&b_name, (3, 1, 1 + (i / MAX_CONNECTIONS) as i32));

			combiner.connect(&a_name, "a_is_bigger");
			combiner.connect(&b_name, "b_is_bigger");
		}

		combiner.connect(format!("and_a/_/{}_0_0", i), &a_name);
		combiner.connect(format!("and_b/_/{}_0_0", i), &b_name);
	}

	combiner.pass_output("a>b", "a_is_bigger", Some("logic")).unwrap();
	combiner.pass_output("a<b", "b_is_bigger", Some("logic")).unwrap();
	combiner.pass_output("a=b", "a_eq_b", Some("logic")).unwrap();

	combiner.pos().place_iter([
		("a", (0, 0, 0)),
		("and_a", (1, 0, 0)),
		("and_b", (3, 0, 0)),
		("b", (4, 0, 0)),

		("a_is_bigger", (1, 0, 1)),
		// ("check_nor", (2, 0, 1)),
		("b_is_bigger", (3, 0, 1)),
		("a_eq_b", (2, 0, 2 + (word_size / MAX_CONNECTIONS) as i32)),
	]);

	combiner.pos().rotate_iter(
		[
			"a", "b", "and_a", "and_b", // "diff_xor", "check_nor"
		].into_iter().map(|name| (name, (0, 0, 1)))
	);

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

// Divide algo

//	Set remainder to a
// For i in 0..word_size
// if b << i > rem
// 		rem -= b << i
//		result |= 1 << i

pub fn divider(bits_before_point: u32, bits_after_point: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	let _thread_delay = 4;
	let word_size = bits_before_point + bits_after_point;

	combiner.add("a", input_filter_rational(bits_before_point, bits_after_point)).unwrap();
	combiner.pass_input("a", "a", None as Option<String>).unwrap();
	combiner.pass_input("a_rational", "a/rational", None as Option<String>).unwrap();

	combiner.add("b", input_filter_rational(bits_before_point, bits_after_point)).unwrap();
	combiner.pass_input("b", "b", None as Option<String>).unwrap();
	combiner.pass_input("b_rational", "b/rational", None as Option<String>).unwrap();

	combiner.add("remainder", adder_compact(word_size)).unwrap();
	{
		// combiner.add("rem_reset", );
		combiner.line_rot_mul(["rem_cycle_1", "rem_cycle_2"], AND, word_size).unwrap();

		combiner.connect("remainder", "rem_cycle_1");
		combiner.connect("rem_cycle_1", "rem_cycle_2");
		combiner.connect("rem_cycle_2", "remainder/b");
	}

	combiner.add("inverter", inverter(word_size)).unwrap();
	combiner.add("compare", fast_compare(word_size)).unwrap();

	let activators_count = ((word_size + MAX_CONNECTIONS - 1) / MAX_CONNECTIONS) as i32;

	combiner.pos().place_iter([
		("a", (0, -activators_count, 0)),
		("b", (0, -activators_count, 1)),
		("remainder", (3, 0, 0)),
		("rem_cycle_1", (5, 0, 1)),
		("rem_cycle_2", (2, 0, 1)),
	]);

	combiner.pos().rotate_iter(
		["rem_cycle_1", "rem_cycle_2"]
			.map(|x| (x, (0, 0, 1)))
	);

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}
