use std::fs;
fn main() {
    // let it panic if the file does not exist
    let contents: String = fs::read_to_string("static/fib.txt").unwrap();
    let numbers: Vec<u64> = Vec::with_capacity(3);
}
