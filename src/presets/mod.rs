use crate::bind::Bind;
use crate::combiner::Combiner;
use crate::positioner::ManualPos;
use crate::scheme::Scheme;
use crate::shape::Shape;
use crate::util::{Bounds, Point, Rot};

pub mod math;

// Basic math:
// adder - done
// inverter - done
// multiplier - done
// divider
// thread adder
// multiplier on thread adder

// Convertors:
// Binary to decimal
// Decimal to binary
// Table bintodec
// Table dectobin
// ANSI table
// ANSI + Rus Table

// Memory:
// Simple XOR memory cell
// Queue + bidirectional queue
// Stack
// Array
// Vector

// Display:
// Number display (customizable paddings)
// Small symbol display
// Full symbol display
// Graphics display (matrix symbol)
// Graphics display from N tables

// Misc:
// Number table generator
// Bool table generator

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

