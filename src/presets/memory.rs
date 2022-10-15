use crate::combiner::Combiner;
use crate::presets::{binary_selector, Scheme};
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
	// NOT FULLY DONE YET
	let mut combiner = Combiner::pos_manual();
	let cell = smallest_rw_cell(word_size);
	let cell_size: (i32, i32, i32) = cell.bounds().cast().tuple();
	let cells_count = size.0 * size.1 * size.2;
	let address_size = (cells_count as f64).log2().ceil() as u32;

	let mut all_cells: Vec<String> = vec![];
	for x in 0..size.0 {
		for y in 0..size.1 {
			for z in 0..size.2 {
				let name = format!("{}_{}_{}", x, y, z);
				combiner.add(name.clone(), cell.clone()).unwrap();
				combiner.pos().place_last((
					x as i32 * cell_size.0,
					y as i32 * cell_size.1,
					z as i32 * cell_size.2
				));

				all_cells.push(name);
			}
		}
	}

	let cell_selector = binary_selector(address_size);
	combiner.add("address", cell_selector).unwrap();
	combiner.pos().place_last((-3, 0, 0));
	combiner.pass_input("address", "address", Some("binary")).unwrap();

	for (i, cell) in all_cells.iter().enumerate() {
		combiner.connect(format!("address/{}", i), format!("{}/activate", cell));
	}

	let (scheme, invalid) = combiner.compile().unwrap();
	println!("Invalid conns: {:?}", invalid.connections);
	scheme
}

// Smallest read/write memory cell
// Inputs: activate, write
// Outputs: _ (data)
pub fn smallest_rw_cell(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add_shapes_cube("input", (word_size, 1, 1), AND, Facing::NegY.to_rot()).unwrap();
	combiner.add_shapes_cube("output", (word_size, 1, 1), AND, Facing::NegY.to_rot()).unwrap();
	combiner.add_shapes_cube("memory", (word_size, 1, 1), XOR, Facing::NegY.to_rot()).unwrap();

	combiner.connect("input", "memory");
	combiner.connect_iter(["memory"], ["output", "memory"]);
	combiner.dim_iter(["activate"], ["input", "output"], (true, true, true));

	combiner.add("activate", AND).unwrap();
	combiner.pos().rotate_last(Facing::NegY.to_rot());

	combiner.pass_input("activate", "activate", Some("logic")).unwrap();
	combiner.pass_input("data", "input", Some("_")).unwrap();
	combiner.pass_output("_", "output", Some("_")).unwrap();

	combiner.pos().place_iter([
		("activate", (1, 0, word_size as i32)),
		("input", (0, 0, 0)),
		("output", (1, 0, 0)),
		("memory", (2, 0, 0)),
	]);

	combiner.pos().rotate_iter([
		("input", (0, -1, 0)),
		("output", (0, -1, 0)),
		("memory", (0, -1, 0)),
	]);

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}