use std::time::Instant;
use wrapper_small_pot::contribute_with_file;

fn main() {
    println!("Hello, wrapper-small-pot!");
    let start = Instant::now();
    contribute_with_file(
        "initialContribution.json",
        "finalContribution.json",
        ["entropy","generated","by", "participant"]
    ).unwrap();
    println!("total time: {:?}", start.elapsed());
}