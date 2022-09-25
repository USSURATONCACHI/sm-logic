use crate::util::mat3::Mat3x3;
use super::*;

#[test]
fn it_works() {
	let mat1: Mat3x3<i32> = Mat3x3::new(0);
	let mat2: Mat3x3<i32> = Mat3x3::new(1);

	println!("Mat1: {:?}", mat1 + mat2);
	assert_eq!(true, false);
}