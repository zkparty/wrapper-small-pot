use std::time::Instant;
use wrapper_small_pot::{
    contribute_with_file,
    check_subgroup_with_file,
    verify_update_with_file,
    get_pot_pubkeys,
};

fn main() {
    println!("Hello, wrapper-small-pot!");

    let in_path = "initialContribution.json";
    let out_path = "updatedContribution.json";
    let proof_path = "updateProofs.json";
    let string_secrets = [
        "0xaabbccddeeff001122334455",
        "0x001122334455667788aa99bb",
        "0x0a1b2c3d4e5f0a1b2c3d4e5f",
        "0x0a9a8a7b6c5c4d3f11223344",
    ];

    let pot_pubkeys = get_pot_pubkeys(string_secrets).unwrap();
    println!("{:?}", pot_pubkeys[0]);
    /*
    println!("subgroup check with file initialized");
    let start_subgroup_check = Instant::now();
    check_subgroup_with_file(in_path).unwrap();
    println!("subgroup check time: {:?}", start_subgroup_check.elapsed());


    println!("contribute with file initialized");
    let start_contribute = Instant::now();
    contribute_with_file(
        in_path,
        out_path,
        proof_path,
        string_secrets,
    ).unwrap();
    println!("contribute time: {:?}", start_contribute.elapsed());


    println!("verify update with file initialized");
    let start_verify_update = Instant::now();
    verify_update_with_file(
        in_path,
        out_path,
        proof_path,
        string_secrets,
    ).unwrap();
    println!("verify update time: {:?}", start_verify_update.elapsed());
    */
}