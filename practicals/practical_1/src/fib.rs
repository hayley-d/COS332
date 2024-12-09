use std::collections::VecDeque;
use std::fs;

fn main_fun() {
    let file_contents: String =
        fs::read_to_string("static/fib.txt").expect("Unable to find file: static/fib.txt");

    let mut numbers: VecDeque<u64> = VecDeque::with_capacity(3);

    for line in file_contents.lines() {
        numbers.push_back(line.trim().parse::<u64>().expect("Error parsing numbers"));
    }
}

fn print_html(numbers: &Vec<u64>) {}

fn prev(numbers: &mut VecDeque<u64>) {
    let prev_sequence: u64 = numbers[1] - numbers[0];
    numbers.pop_back();
    numbers.push_back(prev_sequence);
}

fn next(numbers: &mut VecDeque<u64>) {
    let next_sequence: u64 = numbers[1] + numbers[2];
    numbers.pop_front();
    numbers.push_back(next_sequence);
}
