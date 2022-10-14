use crate::bind::Bind;
use crate::combiner::Combiner;
use crate::presets::Scheme;
use crate::shape::vanilla::GateMode::*;
use crate::util::Facing;

// Inputs: data, write
// Outputs: _ (data)
pub fn xor_mem_cell(size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add("write", OR).unwrap();

	combiner.add_shapes_cube("compare", (size, 1, 1), XOR, Facing::PosY.to_rot()).unwrap();
	combiner.add_shapes_cube("approve", (size, 1, 1), AND, Facing::PosY.to_rot()).unwrap();
	combiner.add_shapes_cube("memory", (size, 1, 1), XOR, Facing::PosZ.to_rot()).unwrap();

	combiner.connect_iter(["memory"], ["memory", "compare"]);
	combiner.connect("compare", "approve");
	combiner.connect("approve", "memory");
	combiner.dim("write", "approve", (true, true, true));


	combiner.pos().place_iter([
		("compare", (0, 0, 0)),
		("approve", (0, 0, 1)),
		("memory", (0, 0, 2)),
		("write", (0, size as i32, 1))
	]);
	combiner.pos().rotate_iter([
		("compare", (0, 0, 1)),
		("approve", (0, 0, 1)),
		("memory", (0, 0, 1)),
	]);

	combiner.pass_input("data", "compare", Some("_")).unwrap();
	combiner.pass_input("write", "write", Some("logic")).unwrap();
	combiner.pass_output("_", "memory", Some("_")).unwrap();

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

pub fn array(word_size: u32, size: (u32, u32, u32)) -> Scheme {
	let mut combiner = Combiner::pos_manual();
	let cell = smallest_rw_cell(word_size);
	let cell_size: (i32, i32, i32) = cell.bounds().cast().tuple();
	println!("Cell size: {:?} | {:?}", cell_size, cell.calculate_bounds().1);

	for x in 0..size.0 {
		for y in 0..size.1 {
			for z in 0..size.2 {
				combiner.add(format!("{}_{}_{}", x, y, z), cell.clone()).unwrap();
				combiner.pos().place_last((
					x as i32 * cell_size.0,
					y as i32 * cell_size.1,
					z as i32 * cell_size.2
				));
			}
		}
	}

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

// Smallest read/write memory cell
// Inputs: activate, write
// Outputs: _ (data)
pub fn smallest_rw_cell(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	let mut input_bind = Bind::new("write", "binary", (word_size, 1, 1));
	let mut output_bind = Bind::new("_", "binary", (word_size, 1, 1));

	for i in 0..word_size {
		let input_name = format!("{}_input", i);
		combiner.add(&input_name, AND).unwrap();
		combiner.pos().place_last((0, 0, i as i32));
		combiner.pos().rotate_last(Facing::NegY.to_rot());
		combiner.connect("activate", &input_name);

		input_bind.connect(((i as i32, 0, 0), (1, 1, 1)), &input_name);

		let output_name = format!("{}_output", i);
		combiner.add(&output_name, AND).unwrap();
		combiner.pos().place_last((1, 0, i as i32));
		combiner.pos().rotate_last(Facing::NegY.to_rot());
		combiner.connect("activate", &output_name);

		output_bind.connect(((i as i32, 0, 0), (1, 1, 1)), &output_name);

		let xor_name = format!("{}_xor", i);
		combiner.add(&xor_name, XOR).unwrap();
		combiner.pos().place_last((2, 0, i as i32));
		combiner.pos().rotate_last(Facing::NegY.to_rot());
		combiner.connect(&xor_name, &xor_name);
		combiner.connect(&input_name, &xor_name);
		combiner.connect(&xor_name, &output_name);
	}

	combiner.add("activate", AND).unwrap();
	combiner.pos().place_last((1, 0, word_size as i32));
	combiner.pos().rotate_last(Facing::NegY.to_rot());
	combiner.pass_input("activate", "activate", Some("logic")).unwrap();

	combiner.bind_input(input_bind).unwrap();
	combiner.bind_output(output_bind).unwrap();

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}