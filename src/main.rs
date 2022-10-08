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
        "0x6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",
        "0xd4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35",
        "0x4e07408562bedb8b60ce05c1decfe3ad16b72230967de01f640b7e4729b49fce",
        "0x4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a",
    ];



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

    println!("get potPubkeys from secrets initialized");
    let start_get_pot_pubkeys = Instant::now();
    let pot_pubkeys = get_pot_pubkeys(string_secrets).unwrap();
    println!("{:?}", pot_pubkeys);
    println!("get potPubkeys time: {:?}", start_get_pot_pubkeys.elapsed());
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_to_pubkey_test() {
        // This test ensures that pubkeys dericvation appears correct
        let string_secrets = [
            "0x6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",
            "0xd4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35",
            "0x4e07408562bedb8b60ce05c1decfe3ad16b72230967de01f640b7e4729b49fce",
            "0x4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a",
        ];

        let pot_pubkeys = get_pot_pubkeys(string_secrets).unwrap();
        println!("{:?}", pot_pubkeys[0]);
        println!("{:?}", pot_pubkeys[1]);
        println!("{:?}", pot_pubkeys[2]);
        println!("{:?}", pot_pubkeys[3]);

        assert_eq!(pot_pubkeys[0].get(0..2).unwrap(), String::from("0x"));
        assert_eq!(pot_pubkeys[0].len(), 194);
    }
}