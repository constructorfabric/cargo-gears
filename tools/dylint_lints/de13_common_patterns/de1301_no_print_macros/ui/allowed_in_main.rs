// compile-flags: --crate-type=bin

fn helper() {
    println!("allowed in binary crate helper");
    eprintln!("allowed in binary crate helper");
}

fn main() {
    helper();
    println!("hello");
    eprintln!("hello");
    print!("hello");
    eprint!("hello");
    dbg!(42);
}
