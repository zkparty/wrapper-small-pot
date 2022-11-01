use std::time::Instant;
use wrapper_small_pot::{
    contribute_with_file,
    check_subgroup_with_file,
    get_pot_pubkeys_with_string,
};

fn main() {
    println!("Hello, wrapper-small-pot!");

    let in_path = "wasm/initialContribution.json";
    let out_path = "wasm/updatedContribution.json";
    let string_identity = "eth|0x000000000000000000000000000000000000dead";
    let string_secret = "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b";

    println!("get potPubkeys from secret initialized");
    let start_get_pot_pubkeys = Instant::now();
    let pot_pubkeys = get_pot_pubkeys_with_string(string_secret).unwrap();
    println!("{:?}", serde_json::to_value(pot_pubkeys).unwrap());
    println!("get potPubkeys time: {:?}", start_get_pot_pubkeys.elapsed());

    println!("contribute with file initialized");
    let start_contribute = Instant::now();
    contribute_with_file(
        in_path,
        out_path,
        string_secret,
        string_identity,
    ).unwrap();
    println!("contribute time: {:?}", start_contribute.elapsed());


    println!("subgroup check with file initialized");
    let start_subgroup_check = Instant::now();
    check_subgroup_with_file(in_path).unwrap();
    println!("subgroup check time: {:?}", start_subgroup_check.elapsed());

}