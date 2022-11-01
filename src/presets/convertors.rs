use std::collections::HashMap;
use crate::bind::Bind;
use crate::combiner::Combiner;
use crate::presets::math::adder_mem;
use crate::presets::shift_connection;
use crate::scheme::Scheme;
use crate::shape::vanilla::GateMode::*;
use crate::shape::vanilla::Timer;
use crate::util::Facing;

// TODO: Make bindec_array slots 2D instead of 1D

/// ***Inputs***: _ (binary number).
///
/// ***Outputs***: all, 0, 1, 2, 3, etc... (one for each bindec digit).

///
/// Converts binary numbers to decimal (bindec) numbers. Each decimal
/// digit is represented as one 4-bit binary number. There is different
/// output for each decimal digit.
///
/// Time complexity: `O(word_size)`.
/// Space complexity: `O(word_size.pow(2))`
///
/// ***Bindec digits examples*** (base 2 -> base 10):<br>
/// 0b0000 -> 0<br>
/// 0b0001 -> 1<br>
/// 0b0010 -> 2<br>
/// 0b0011 -> 3<br>
/// 0b0100 -> 4<br>
/// 0b0101 -> 5<br>
/// 0b0110 -> 6<br>
/// 0b0111 -> 7<br>
/// 0b1000 -> 8<br>
/// 0b1001 -> 9<br>
pub fn bin_to_bindec(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add_shapes_cube("input", (word_size, 1, 1), OR, (0, 0, 0)).unwrap();
	combiner.pos().rotate_last((0, 0, 1));
	let mut input = Bind::new("_", "binary", (word_size, 1, 1));
	input.connect_full("input");
	input.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	combiner.bind_input(input).unwrap();

	let section = add_3_if_more_th_4();

	let mut bits_matrix: Vec<Vec<Option<String>>> = vec![]; // bits_matrix[y][x]
	let max_sections_count = word_size as i32 - 3;
	combiner.pos().place("input", (-max_sections_count, 0, 1));

	let mut col = 0;
	let mut sections_start: i32 = 1;
	let mut sections_end: i32 = word_size as i32 + 1;
	// Sections
	'column: loop {
		let sections_count = (sections_end - sections_start - 3).max(0);
		if sections_count == 0 {
			break 'column;
		}

		for row in 0..sections_count {
			if bits_matrix.get(row as usize).is_none() {
				bits_matrix.push(vec![]);
			}

			let name = format!("section_{}_{}", col, row);

			combiner.add(&name, section.clone()).unwrap();
			let start = sections_start + row;

			combiner.pos().place_last((-row, start, 0));

			for _ in (bits_matrix[row as usize].len() as i32)..start {
				bits_matrix[row as usize].push(None);
			}

			for bit in 0..4 {
				let bit_path = format!("{}/_/{}", name, bit);
				bits_matrix[row as usize].push(Some(bit_path));
			}
		}

		col += 1;
		sections_start += 4;
		sections_end += 1;
	}

	// Connect sections top-down
	let mut prev_row: HashMap<usize, String> = (0..word_size)
		.map(|x| (x as usize, format!("input/_/{}_0_0", x)))
		.collect();

	bits_matrix.reverse();
	for row in bits_matrix {
		for (i, point) in row.into_iter().enumerate() {
			let point = match point {
				None => continue,
				Some(point) => point,
			};

			// Try to connect from prev row
			match prev_row.get(&i) {
				None => {}
				Some(connect_from) => combiner.connect(connect_from.clone(), point.clone()),
			}
			prev_row.insert(i, point);
		}
	}

	// Bind outputs from sections
	let mut prev_row: Vec<(usize, String)> = prev_row.into_iter().collect();
	prev_row.sort_by(|(a, _), (b, _)| a.cmp(b));

	fn connect_output_to(output: &mut Bind, bit_id: usize, to: Option<String>) {
		match to {
			None => {}
			Some(to) => {
				output.connect(((bit_id as i32, 0, 0), (1, 1, 1)), to);
			}
		}
	}

	let mut output_id = 0;
	let mut all_outputs = Bind::new("all", "bindec_array", (prev_row.len() as u32, 1, 1));
	for (i, elem) in &prev_row {
		all_outputs.connect(((*i as i32, 0, 0), (1, 1, 1)), elem);
	}
	combiner.bind_output(all_outputs).unwrap();

	let mut iter = prev_row.into_iter().map(|(_, value)| value);

	'for_4_elems: loop {
		let first_elem = iter.next();
		if first_elem.is_none() {
			break 'for_4_elems;
		}

		let mut output = Bind::new(format!("{}", output_id), "bindec", (4, 1, 1));

		connect_output_to(&mut output, 0, first_elem);
		connect_output_to(&mut output, 1, iter.next());
		connect_output_to(&mut output, 2, iter.next());
		connect_output_to(&mut output, 3, iter.next());

		output.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
		combiner.bind_output(output).unwrap();

		output_id += 1;
	}

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

/// ***Inputs***: _.
///
/// ***Outputs***: _.

///
/// Just a section of for `bin_to_bindec` scheme. If the input is binary
/// number no more than 4, output is the number. Otherwise - output is
/// the number + 3.
pub fn add_3_if_more_th_4() -> Scheme {
	let mut combiner = Combiner::pos_manual();
	let mut input = Bind::new("_", "binary", (4, 1, 1));
	input.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();

	combiner.add_shapes_cube("out_xor", (4, 1, 1), XOR, Facing::PosZ.to_rot()).unwrap();
	combiner.pos().rotate_last((0, 0, 1));
	input.connect_full("out_xor");

	// COMPARE IF MORE THAN 4
	combiner.add_iter([
		("compare_0", OR),
		("compare_1", AND),
		("compare_2", AND),
	]).unwrap();
	input.connect(((3, 0, 0), (1, 1, 1)), "compare_0");
	input.connect(((2, 0, 0), (1, 1, 1)), "compare_1");
	input.connect(((2, 0, 0), (1, 1, 1)), "compare_2");
	input.connect(((1, 0, 0), (1, 1, 1)), "compare_1");
	input.connect(((0, 0, 0), (1, 1, 1)), "compare_2");
	combiner.connect_iter(["compare_1", "compare_2"], ["compare_0"]);

	// ADD 3 (0b0011)
	combiner.connect_iter(["compare_0"], ["out_xor/_/0_0_0", "out_xor/_/1_0_0"]);

	combiner.add("carry_bit_0", AND).unwrap();
	combiner.connect("compare_0", "carry_bit_0");
	input.connect(((0, 0, 0), (1, 1, 1)), "carry_bit_0");
	combiner.connect("carry_bit_0", "out_xor/_/1_0_0");


	combiner.add_iter([
		("carry_bit_1_and_0", AND),
		("carry_bit_1_and_1", AND),
		("carry_bit_1_and_2", AND),
		("carry_bit_1_or", OR),
	]).unwrap();

	combiner.connect_iter(["carry_bit_1_and_0", "carry_bit_1_and_1", "carry_bit_1_and_2"], ["carry_bit_1_or"]);
	combiner.connect_iter(["carry_bit_0"], ["out_xor/1", "carry_bit_1_and_0", "carry_bit_1_and_1"]);
	input.connect(((1, 0, 0), (1, 1, 1)), "carry_bit_1_and_1");
	input.connect(((1, 0, 0), (1, 1, 1)), "carry_bit_1_and_2");
	combiner.connect_iter(["compare_0"], ["carry_bit_1_and_2", "carry_bit_1_and_0"]);
	combiner.connect_iter(["carry_bit_1_or"], ["carry_bit_2", "out_xor/_/2_0_0"]);

	combiner.add("carry_bit_2", AND).unwrap();
	input.connect(((2, 0, 0), (1, 1, 1)), "carry_bit_2");
	combiner.connect("carry_bit_2", "out_xor/_/3_0_0");


	combiner.bind_input(input).unwrap();
	combiner.pos().place_iter([
		("out_xor", (0, 0, 3)),

		("compare_0", (0, 2, 1)),
		("compare_1", (0, 3, 1)),
		("compare_2", (0, 3, 0)),

		("carry_bit_0", (0, 0, 1)),
		("carry_bit_1_and_0", (0, 0, 2)),
		("carry_bit_1_and_1", (0, 1, 2)),
		("carry_bit_1_and_2", (0, 2, 2)),
		("carry_bit_1_or", (0, 3, 2)),
		("carry_bit_2", (0, 1, 1)),
		("", (0, 0, 0)),
	]);

	let mut output = Bind::new("_", "binary", (4, 1, 1));
	output.connect_full("out_xor");
	output.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
	combiner.bind_output(output).unwrap();

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

/// ***Inputs***: start, all, 0, 1, 2, 3, etc... (one for each bindec digit).
///
/// ***Outputs***: _.

///
/// Converts decimal (bindec) number to binary. After data is set to
/// all the digits, 1-tick signal needs to be sent to 'start' input.
/// `4 * digits_count * 3 + ~adder_mem_delay` ticks later result will be
/// available.
///
/// Time complexity: `O(digits_count)`
/// Space complexity: `O(digits_count)`
pub fn bindec_to_bin(digits_count: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add("start", OR).unwrap();
	combiner.add_shapes_cube("input", (digits_count * 4, 1, 1), OR, Facing::PosY.to_rot()).unwrap();
	combiner.add_shapes_cube("and", (digits_count * 4, 1, 1), AND, Facing::PosY.to_rot()).unwrap();
	combiner.add_shapes_cube("timer", (digits_count * 4, 1, 1), Timer::new(2), Facing::PosZ.to_rot()).unwrap();

	combiner.connect("input", "and");
	combiner.connect("timer", "and");
	combiner.custom("timer", "timer", shift_connection((1, 0, 0)));
	combiner.connect("start", "timer");

	let bits_count = (digits_count as f64 * 10_f64.log2()).ceil() as u32;

	combiner.add("adder", adder_mem(bits_count)).unwrap();
	combiner.connect("start", "adder/reset");

	for digit in 0..digits_count {
		for bit in 0..4 {
			let digit_binary_equivalent = 10_u128.pow(digit) * (1 << bit);
			let global_bit_id = digit * 4 + bit as u32;

			for result_bit_id in 0..128 {
				if digit_binary_equivalent & (1 << result_bit_id) > 0 {
					combiner.connect(format!("and/_/{}_0_0", global_bit_id), format!("adder/_/{}", result_bit_id));
				}
			}
		}
	}

	combiner.pass_output("_", "adder", None as Option<String>).unwrap();
	combiner.pass_input("start", "start", Some("logic")).unwrap();
	let mut all_inputs = Bind::new("all", "bindec_array", (digits_count * 4, 1, 1));
	all_inputs.connect_full("input");
	combiner.bind_input(all_inputs).unwrap();

	for i in 0..digits_count {
		let mut input = Bind::new(format!("{}", i), "bindec", (4, 1, 1));
		input.connect_func(|x, _y, _z| Some(format!("input/_/{}_0_0", x as u32 + i * 4)));
		input.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();
		combiner.bind_input(input).unwrap();
	}


	combiner.pos().place_iter([
		("input", (0, 0, 0)),
		("and", (0, 0, 1)),
		("timer", (0, 0, 2)),
		("start", (0, 0, 4)),
		("adder", (1, 0, 0)),
	]);
	combiner.pos().rotate_iter([
		("input", (0, 0, 1)),
		("and", (0, 0, 1)),
		("timer", (0, 0, 1)),
	]);
	combiner.pos().rotate("start", Facing::NegX.to_rot());

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}