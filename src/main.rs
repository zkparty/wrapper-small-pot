use std::time::Instant;
use wrapper_small_pot::{contribute_with_file, check_subgroup_with_file};

fn main() {
    println!("Hello, wrapper-small-pot!");

    println!("subgroup check initialized");
    let start_subgroup_check = Instant::now();
    check_subgroup_with_file("initialContribution.json").unwrap();
    println!("subgroup check time: {:?}", start_subgroup_check.elapsed());

    println!("contribute with file initialized");
    let start_contribute = Instant::now();
    contribute_with_file(
        "initialContribution.json",
        "updatedContribution.json",
        ["0xdeadbeef","0x001122334455667788aa99bb","0x0a1b2c3d4e5f", "0x0a9a8a7b6c5c4d3f"]
    ).unwrap();
    println!("contribute time: {:?}", start_contribute.elapsed());
}