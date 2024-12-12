use std::collections::VecDeque;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();

    let increase_sequence: bool = match &args[1][..] {
        "next" => true,
        _ => false,
    };

    let file_contents: String =
        fs::read_to_string("static/fib.txt").expect("Unable to find file: static/fib.txt");

    let mut numbers: VecDeque<u64> = VecDeque::with_capacity(3);

    for line in file_contents.lines() {
        numbers.push_back(line.trim().parse::<u64>().expect("Error parsing numbers"));
    }

    if increase_sequence {
        next(&mut numbers);
    } else {
        prev(&mut numbers);
    }

    let updated_numbers: String = format!("{}\n{}\n{}", numbers[0], numbers[1], numbers[2]);
    match fs::write("static/fib.txt", updated_numbers) {
        Ok(_) => (),
        Err(_) => eprintln!("Error updating fib file"),
    }

    print_html(&numbers);
}

fn print_html(numbers: &VecDeque<u64>) {
    println!("<!DOCTYPE html>");
    println!("<html lang=\"en\">");
    println!("<head>");
    println!("<meta charset=\"UTF-8\">");
    println!("<meta http-equiv=\"Cache-Control\" content=\"no-store\">");
    println!("<title>Fibonacci</title>");
    println!("</head>");
    println!("<body>");
    println!("<h1>Fibonacci Numbers</h1>");
    println!(
        "<p>Fibonacci numbers: {}, {}, {}</p>",
        numbers.get(0).unwrap(),
        numbers.get(1).unwrap(),
        numbers.get(2).unwrap()
    );
    println!("<a href=\"/fib/next\"><button>Next</button></a>");
    println!("<a href=\"/fib/prev\"><button>Previous</button></a>");
    println!("</body>");
    println!("</html>");
}

fn prev(numbers: &mut VecDeque<u64>) {
    if numbers[0] == 0 {
        numbers.clear();
        numbers.push_back(0);
        numbers.push_back(1);
        numbers.push_back(1);
    } else {
        numbers.pop_back();
        numbers.push_front(numbers[1] - numbers[0]);
    }
}

fn next(numbers: &mut VecDeque<u64>) {
    numbers.pop_front();
    numbers.push_back(numbers[0] + numbers[1]);
}
