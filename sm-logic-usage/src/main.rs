use sm_logic::shape::vanilla::Gate;
use sm_logic::shape::vanilla::Timer;
use sm_logic::util::GateMode;

fn main() {
	let gate = Gate::new(GateMode::AND);
	let timer = Timer::from_time(17, 18);

	println!("Gate: {:?}", gate);
	println!("Timer: {:?}", timer);
}