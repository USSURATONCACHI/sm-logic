use sm_logic::combiner::*;
use sm_logic::util::GateMode::{AND, OR};

fn main() {
	let mut combiner = Combiner::pos_manual();

	combiner.add("a", AND).unwrap();
	combiner.pos().place_last((0, 0, 0));

	combiner.add("b", OR).unwrap();
	combiner.pos().place_last((0, 0, 1));

	combiner.connect_iter(["a"], ["a", "b"]);
	combiner.connect("b", "a");

	match combiner.compile() {
		Err(e) => println!("Fail: {:?}", e),
		Ok((scheme, invalid_acts)) => {
			println!("Invalid: {:?}", invalid_acts);
			println!("Success.");
			println!("Shapes count: {}", scheme.shapes_count());
			println!("Inputs:");
			for inp in scheme.inputs() {
				println!("\t{} - {:?}", inp.name(), inp.bounds().tuple());
			}

			println!("\nOutputs:");
			for inp in scheme.outputs() {
				println!("\t{} - {:?}", inp.name(), inp.bounds().tuple());
			}

			println!("\nShapes:");
			for (pos, _, shape) in scheme.shapes() {
				println!("\tpos {:?} - {:?}", pos.tuple_ref(), shape);
			}
		}
	}
}