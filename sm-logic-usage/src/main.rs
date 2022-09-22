fn main() {
    println!("Hello, world!");
    // Testing traits
    let a = TypeA {};
    a.print_some();
    let b = TypeB {};
    b.print_some();

    let a: Box<dyn Test> = Box::new(a);
    let b: Box<dyn Test> = Box::new(b);

    a.print_some();
    b.print_some();
}

trait Test {
    fn print_some(&self) {
        println!("Something...");
    }
}

pub struct TypeA {}

impl Test for TypeA {
    fn print_some(&self) {
        println!("Printing from impl TypeA!");
    }
}

pub struct TypeB {}

impl Test for TypeB {}