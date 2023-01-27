use crate::bind::Bind;
use crate::combiner::{Combiner, Error};
use crate::connection::{Connection, ConnMap};
use crate::positioner::{ManualPos, Positioner};
use crate::scheme::Scheme;
use crate::shape::Shape;
use crate::shape::vanilla::GateMode::*;
use crate::util::{Bounds, Facing, MAX_CONNECTIONS, Point, Rot};

pub mod math;
pub mod memory;
pub mod convertors;
pub mod display;

// Basic math:
// adder - done
// inverter - done
// multiplier - done
// thread adder - done
// multiplier on thread adder - done
// divider
// fastest counter
// counter that has +1 and -1
// xor memory cleaning module

// Memory:
// Simple XOR memory cell - done
// Array - done
// Shift memory arrays - done

// Convertors:
// Binary to decimal - done
// Decimal to binary - done
// Table bintodec - omitted
// Table dectobin - omitted

// Display:
// Number display (customizable paddings)
// Small symbol display
// Full symbol display
// Graphics display (matrix symbol)
// Graphics display from N tables

// Misc:
// Number table generator
// Bool table generator
// Binary selector - done

/// Creates `Bind` of slot, that contains binary number splitted in two
/// parts.
///
/// ***Sectors***:
///
/// ...3, 2, 1, 0, -1, -2, -3, -4... - for bits of number. Names
/// represent power of bit. All negative names are for fractional bits
/// (1/2, 1/4 and so on).
///
/// integer - for integer part of number.
///
/// fractional - for fractional part of number (reverse order of bits).
///
/// integer/0, integer/1, integer/2...
///
/// fractional/0, fractional/1, fractional/2...
///
/// for each bit of corresponding part.
///
/// ***Sectors end***
///
/// Integer part is stored in sector 'integer' (usage: '%name%/integer').
/// Integer bits are laid in points (bit_id, 0, 0) of abstract slot space.
///
/// Fractional part is stored in sector 'fractional' in reverse order
/// (usage: '%name%/fractional').
/// Fractional bits are laid in points (bit_id, 1, 0).
///
/// Arguments:
///
/// `name` - name of the slot/bind.
///
/// `target` - name of the line of gates, where bits are stored
/// initially as just plain binary number.
///
/// `bits_before_point` - amount of integer bits.
///
/// `bits_after_point` - amount of fractional bits.
///
/// `shift_to_integer` - position of the first integer bit on the `target`.
///
/// `shift_to_fractional` - position of the first fractional bit on the `target`.
///
pub fn make_rational_bind<S1: Into<String>, S2: Into<String>>(
	name: S1, target: S2,
	bits_before_point: u32, bits_after_point: u32,
	shift_to_integer: u32, shift_to_fractional: u32,
) -> Bind {
	let bind_size = bits_before_point.max(bits_after_point);
	let mut rational = Bind::new(name, "binary.rational", (bind_size, 2, 1));

	let target = target.into();
	let integer = shift_connection(((shift_to_integer as i32), 0, 0));
	rational.custom(((0, 0, 0), (bits_before_point, 1, 1)), &target, integer);

	// |x, size| size - x - 1
	let fractional = ConnMap::new(move |(point, _), _| {
		let (x, y, z) = point.tuple();
		Some(Point::new(bits_after_point as i32 - x - 1 + shift_to_fractional as i32, y, z))
	});
	rational.custom(((0, 1, 0), (bits_after_point, 1, 1)), &target, fractional);

	// integer
	rational.add_sector("integer", (0, 0, 0), (bits_before_point, 1, 1), "binary").unwrap();
	for i in 0..bits_before_point {
		rational.add_sector(format!("integer/{}", i), (i as i32, 0, 0), (1, 1, 1), "bit").unwrap();
		rational.add_sector(format!("{}", i), (i as i32, 0, 0), (1, 1, 1), "bit").unwrap();
	}

	// fractional
	rational.add_sector("fractional", (0, 1, 0), (bits_after_point, 1, 1), "binary.fractional").unwrap();
	for i in 0..bits_after_point {
		rational.add_sector(format!("fractional/{}", i), (i as i32, 1, 0), (1, 1, 1), "bit").unwrap();
		rational.add_sector(format!("{}", -(i as i32) - 1), (i as i32, 1, 0), (1, 1, 1), "bit").unwrap();
	}

	rational
}

pub fn connect_safe<P, T, N, S>(
	combiner: &mut Combiner<P>,
	targets: T,
	mut create_activator: N,
	start_with: Option<String>,
	rev: bool
) -> Result<u32, Error>
	where T: IntoIterator, <T as IntoIterator>::Item: Into<String>,
			N: FnMut(&mut Combiner<P>, u32) -> S,
			S: Into<String>, P: Positioner
{
	let mut activators_count = 0;

	let has_start = start_with.is_some();
	let mut activator = match start_with {
		Some(start) => start,
		None => "none".to_string(),
	};

	for (i, target) in targets.into_iter().enumerate() {
		let should_change = if has_start {
			i != 0 && i % (MAX_CONNECTIONS as usize) == 0
		} else {
			i % (MAX_CONNECTIONS as usize) == 0
		};

		if should_change {
			activator = create_activator(combiner, activators_count).into();
			activators_count += 1;
		}

		if !rev {
			combiner.connect(&activator, target);
		} else {
			combiner.connect(target, &activator);
		}
	}

	Ok(activators_count)
}

/// ***Inputs***: data, activator.
///
/// ***Outputs***: _ (filter).
fn input_filter(size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add_shapes_cube("in_data", (size, 1, 1), OR, Facing::PosZ.to_rot()).unwrap();
	combiner.add_shapes_cube("filter", (size, 1, 1), AND, Facing::PosZ.to_rot()).unwrap();

	let mut activator = Bind::new("activator", "logic", (1, 1, 1));
	combiner.connect("in_data", "filter");
	connect_safe(
		&mut combiner,
		(0..size).map(|i| format!("filter/_/{}_0_0", i)),
		|combiner, i| {
			let name = format!("{}", i);
			combiner.add(&name, OR).unwrap();
			activator.connect_full(&name);
			combiner.pos().place_last((1, -(i as i32) - 1, 0));

			name
		},
		None,
		false
	).unwrap();

	combiner.pos().place("in_data", (0, 0, 0));
	combiner.pos().place("filter", (1, 0, 0));

	combiner.pos().rotate("in_data", (0, 0, 1));
	combiner.pos().rotate("in_data", (0, -1, 0));
	combiner.pos().rotate("filter", (0, 0, 1));

	combiner.bind_input(activator).unwrap();
	combiner.pass_input("data", "in_data", Some("_")).unwrap();
	combiner.pass_output("_", "filter", Some("_")).unwrap();

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

/// ***Inputs***: _, activator, rational.
///
/// ***Outputs***: _ (filter), rational.
fn input_filter_rational(bits_before_point: u32, bits_after_point: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	let word_size = bits_before_point + bits_after_point;
	combiner.add("filter", input_filter(word_size)).unwrap();
	combiner.pos().place_last((0, 0, 0));

	let mut input_def = Bind::new("_", "binary", (word_size, 1, 1));
	input_def.connect_full("filter/data");
	input_def.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();

	let input_rational = make_rational_bind("rational", "filter/data", bits_before_point, bits_after_point, bits_after_point, 0);

	let mut output_def = Bind::new("_", "binary", (word_size, 1, 1));
	output_def.connect_full("filter");
	output_def.gen_point_sectors("bit", |x, _y, _z| x.to_string()).unwrap();

	let output_rational = make_rational_bind("rational", "filter", bits_before_point, bits_after_point, bits_after_point, 0);

	combiner.pass_input("activator", "filter/activator", Some("logic")).unwrap();
	combiner.bind_input(input_def).unwrap();
	combiner.bind_input(input_rational).unwrap();

	combiner.bind_output(output_def).unwrap();
	combiner.bind_output(output_rational).unwrap();

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

pub fn binary_selector(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	let selector = binary_selector_compact(word_size);
	let outputs: Vec<String> = selector.outputs().iter().map(|slot| slot.name().clone()).collect();

	combiner.add("selector", selector).unwrap();
	combiner.pos().place_last((1, 0, 0));

	for output in outputs {
		if output.eq("_") {
			continue;
		}

		let path = format!("selector/{}", output);
		combiner.pass_output(output, path, None as Option<String>).unwrap();
	}

	combiner.add_shapes_cube("input", (word_size, 1, 1), OR, Facing::PosY.to_rot()).unwrap();
	combiner.pass_input("_", "input", Some("binary")).unwrap();
	combiner.pos().place_last((0, 0, 0));
	combiner.pos().rotate_last((0, 0, 1));
	combiner.connect("input", "selector");

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

pub fn binary_selector_compact(word_size: u32) -> Scheme {
	if word_size >= 30 {
		panic!("Binary selectors for word sizes more than 29 is not supported.");
	}

	let outputs_count = 2_i64.pow(word_size);
	let selectors_count = ((outputs_count as f64) / (2.0 * MAX_CONNECTIONS as f64)).ceil() as u32;

	let mut combiner = Combiner::pos_manual();

	let mut input = Bind::new("_", "binary", (word_size, 1, 1));

	for i in 0..selectors_count {
		combiner.add_shapes_cube(format!("sel_pos_{}", i), (word_size, 1, 1), OR, Facing::PosZ.to_rot()).unwrap();
		combiner.pos().place_last((1, 0, i as i32));
		combiner.pos().rotate_last((0, 0, 1));
		input.connect_full(format!("sel_pos_{}", i));

		combiner.add_shapes_cube(format!("sel_neg_{}", i), (word_size, 1, 1), NOR, Facing::PosZ.to_rot()).unwrap();
		combiner.pos().place_last((2, 0, i as i32));
		combiner.pos().rotate_last((0, 0, 1));
		input.connect_full(format!("sel_neg_{}", i));
	}

	combiner.bind_input(input).unwrap();
	let mut conns_to_positive: Vec<u32> = [0].into_iter().cycle().take(word_size as usize).collect();
	let mut conns_to_negative: Vec<u32> = [0].into_iter().cycle().take(word_size as usize).collect();

	for i in 0..outputs_count {
		let bind_name = format!("{}", i);

		let mut bind = Bind::new(&bind_name, "logic", (1, 1, 1));

		for bit in 0..word_size {
			if get_bit(i, bit) {
				let selector_id = conns_to_positive[bit as usize] / MAX_CONNECTIONS;
				bind.connect_full(format!("sel_pos_{}/_/{}_0_0", selector_id, bit));
				conns_to_positive[bit as usize] += 1;
			} else {
				let selector_id = conns_to_negative[bit as usize] / MAX_CONNECTIONS;
				bind.connect_full(format!("sel_neg_{}/_/{}_0_0", selector_id, bit));
				conns_to_negative[bit as usize] += 1;
			}
		}

		combiner.bind_output(bind).unwrap();
	}

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

fn get_bit(number: i64, bit_id: u32) -> bool {
	((number >> bit_id) & 1) == 1
}

pub fn shapes_cube_combiner<B, S, R>(bounds: B, from_shape: S, shape_rot: R) -> Combiner<ManualPos>
	where B: Into<Bounds>, S: Into<Shape>, R: Into<Rot>
{
	let shape = from_shape.into();
	let has_input = shape.has_input();
	let has_output = shape.has_output();

	let mut shape: Scheme = shape.into();
	let shape_rot = shape_rot.into();
	shape.rotate(shape_rot.clone());

	let bounds: (i32, i32, i32) = bounds.into().cast().tuple();
	let mut combiner = Combiner::pos_manual();
	let mut slot = Bind::new("_", "_", (bounds.0 as u32, bounds.1 as u32, bounds.2 as u32));

	for x in 0..bounds.0 {
		for y in 0..bounds.1 {
			for z in 0..bounds.2 {
				let name = format!("{}_{}_{}", x, y, z);

				combiner.add(&name, shape.clone()).unwrap();

				let pos = Point::new(x, y, z);
				combiner.pos().place_last(pos * shape.bounds().cast());

				slot.connect(((x, y, z), (1, 1, 1)), &name);
				slot.add_sector(name, (x, y, z), (1, 1, 1), "logic").unwrap();
			}
		}
	}

	if has_input {
		combiner.bind_input(slot.clone()).unwrap();
	}
	if has_output {
		combiner.bind_output(slot).unwrap();
	}

	combiner
}

pub fn shapes_cube<B, S, R>(bounds: B, from_shape: S, shape_rot: R) -> Scheme
	where B: Into<Bounds>, S: Into<Shape>, R: Into<Rot>
{
	shapes_cube_combiner(bounds, from_shape, shape_rot).compile().unwrap().0
}

pub fn shift_connection(shift: (i32, i32, i32)) -> Box<dyn Connection> {
	ConnMap::new(move |(point, _in_bounds), _out_bounds| Some(point + Point::from_tuple(shift)))
}
