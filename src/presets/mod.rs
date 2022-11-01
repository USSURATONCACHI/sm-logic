use crate::bind::Bind;
use crate::combiner::Combiner;
use crate::connection::{Connection, ConnMap};
use crate::positioner::ManualPos;
use crate::scheme::Scheme;
use crate::shape::Shape;
use crate::shape::vanilla::GateMode::*;
use crate::util::{Bounds, Facing, MAX_CONNECTIONS, Point, Rot};

pub mod math;
pub mod memory;
pub mod convertors;

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

pub fn binary_selector(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add("selector", binary_selector_compact(word_size)).unwrap();
	combiner.pos().place_last((1, 0, 0));

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