use std::io::Write;

fn main() {
    println!("Hello, world!");
    let mut file = std::fs::File::create("./test.txt").unwrap();

    let _ = file.write_all(b"Sup bitch");
}
