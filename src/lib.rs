use crate::combiner::Combiner;

pub mod util;
pub mod combiner;
pub mod connection;
pub mod scheme;
pub mod slot;
pub mod shape;
pub mod positioner;



#[test]
pub fn tmp() {
	let _combiner = Combiner::pos_manual();
}