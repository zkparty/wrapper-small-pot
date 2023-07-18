#[cfg(target_family = "wasm")]
mod wasm;

use eyre::Result;
use hex::FromHex;
use std::str::FromStr;
use std::{fs::File, path::Path};
use ark_serialize::{Read, Write};
//use serde::{Deserialize, Serialize};
use kzg_ceremony_crypto::{
    G2,
    BLST,
    Secret,
    Identity,
    get_pot_pubkeys,
    BatchContribution,
    Transcript,
    CeremonyError,
    Engine,
    DefaultEngine,
};

/**
 * We'll use this function in the cli
 */
pub fn contribute_with_file(in_path: &str, out_path: &str, string_secret: &str, string_identity: &str) -> Result<()> {
    let json = read_json_file(in_path)?;
    let contribution = contribute_with_string(json, string_secret, string_identity)?;

    write_json_file(out_path, &contribution)
}
/**
 * We'll use this function in the wasm
 */
pub fn contribute_with_string(json: String, string_secret: &str, string_identity: &str) -> Result<String> {
    // parse contribution object
    let mut contribution = serde_json::from_str::<BatchContribution>(&json)
    .expect("Contribution deserialization failed");
    // parse entropy
    let entropy = string_to_entropy(string_secret);
    // parse identity (eth or git)
    let identity = Identity::from_str(string_identity)
    .expect("Identity deserialization failed");

    contribution.add_entropy::<BLST>(&entropy, &identity)
    .expect("Contribution computation failed");
    let result = serde_json::to_string(&contribution)
    .expect("Result Contribution serialization failed");

    Ok(result)
}


/**
 * We'll use this function in the cli
 */
pub fn check_subgroup_with_file(in_path: &str) -> Result<()> {
    let json = read_json_file(in_path)?;
    let result = check_subgroup_with_string(json)?;
    Ok(println!("Subgroup check is correct: {:?}", result))
}
/**
 * We'll use this function in the wasm
 */
pub fn check_subgroup_with_string(json: String) -> Result<bool> {
    // parse contribution object
    let mut contribution = serde_json::from_str::<BatchContribution>(&json)
    .expect("Contribution deserialization failed");

    let result = contribution.validate::<BLST>();

    let is_valid = match result {
        Ok(()) => true,
        Err(error) => {
            println!("{:?}", error);
            false
        },
    };
    Ok(is_valid)
}
/**
 * We'll use this function in the wasm
 */
pub fn get_pot_pubkeys_with_string(string_secret: &str) -> Result<Vec<G2>> {
    let entropy = string_to_entropy(string_secret);
    let pot_pubkeys = get_pot_pubkeys::<BLST>(&entropy);
    Ok(pot_pubkeys)
}


/**
 * Util functions
 */
fn read_json_file(string_path: &str) -> Result<String> {
    let path = Path::new(string_path);
    let mut file = File::open(path)
    .expect("error opening file");
    let mut content = String::new();
    file.read_to_string(&mut content)
    .expect("error reading file");
    Ok(content)
}

fn write_json_file(string_path: &str, content: &str) -> Result<()> {
    let buf = content.as_bytes();
    let path = Path::new(string_path);
    let mut file = File::create(path).expect("error creating file");
    file.write_all(buf).expect("error writing in file");
    Ok(())
}

fn string_to_entropy(string_secret: &str) -> Secret<[u8; 32]> {
    let buffer = <[u8; 32]>::from_hex(string_secret)
    .expect("secret should be a 32 hex string");
    let entropy = Secret::from(buffer);
   entropy
}

/// Verifies that a contribution is included in the transcript
fn verify_inclusion<E: Engine>(t: &Transcript, contrib_idx: usize) -> Result<(), CeremonyError> {
    assert!(contrib_idx < t.witness.products.len());

    // Loop through subsequent witness entries. Do pairing check on each.
    let mut index = contrib_idx;

    while index < t.witness.products.len() {
        // Pairing check: this & prev products, this pubkey
        E::verify_pubkey(
            t.witness.products[index],
            t.witness.products[index - 1],
            t.witness.pubkeys[index],
        )?;

        index += 1;
    }

    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_to_pubkey_test() {
        // This test ensures that pubkeys dericvation appears correct
        let string_secret = "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b";

        let pot_pubkeys = get_pot_pubkeys_with_string(string_secret).unwrap();
        println!("{:?}", serde_json::to_value(&pot_pubkeys[0]).unwrap());
        println!("{:?}", serde_json::to_value(&pot_pubkeys[1]).unwrap());
        println!("{:?}", serde_json::to_value(&pot_pubkeys[2]).unwrap());
        println!("{:?}", serde_json::to_value(&pot_pubkeys[3]).unwrap());

        let value_pot_pubkeys_0 = serde_json::to_string(&pot_pubkeys[0]).unwrap();
        assert_eq!(value_pot_pubkeys_0.get(0..3).unwrap(), String::from("\"0x"));
        assert_eq!(value_pot_pubkeys_0.len(), 194 + 2);
    }

    #[test]
    fn test_verify_inclusion() {
        let json = serde_json::json!({
            "numG1Powers": 4,
            "numG2Powers": 2,
            "powersOfTau": {
                "G1Powers": [
                    "0x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                    "0x962dffcca938bfe9cc11ab949c73e1b742ca2fe2f7122170e7ed8ceaea9cf57c411743a5ac4c48d7405f397b63d36a25",
                    "0xa57913e7354d2bdbb631e7b270ad9b0fd34c8ee177c5f0903024cc1da1221fa65c92ba515473aa248137dfc510d5d4c9",
                    "0xb2581616a7420d485eee433d355540fd2d9f441a7864b168ad0e068abf4772fa2f644a192b3953616f9fbd5ac88dcd64",

                ],
                "G2Powers": [
                    "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                    "0x8bab27ba31974bfd253d0d37e8ec7c580fa5cc9dfd81eb2e6faae3c4b38b3efcf1e9f3513a0b4031662bbabefe656614190d96e2503d3c68d44324722f00d3abb1b4ec0ba7aa1f4a8595487649912f87ea8f6761648425e9b9baa5a46e18f2c9",
                ],
            },
            "witness": {
                "runningProducts": [
                    "0x9938667a3807b2bba879c10272b0470507cb5926784976d34a440f6ef1aa0cad8a6963c790642a5e76e67cb6471ee075",
                    "0x94d3d149a60414d74e1e60fd1bb4d08ffef7860d86e698439ebbea8614d7ece57d32a1f9be83389bdf7c64846af2513d",
                    "0x94d3d149a60414d74e1e60fd1bb4d08ffef7860d86e698439ebbea8614d7ece57d32a1f9be83389bdf7c64846af2513d",
                    "0xa6ed552088642b976df969e47dc6503e33450744de1b358cafdd610dda3edbfc92efc98c77a4960010a89fa60d0dd127",
                    "0xb4d01655876e28edfa71521ab9fc5d916d9f3ea1c51477c7f912501246a9a7643d2f4ae971563e98dfbdd28df764bedd",
                    "0x962dffcca938bfe9cc11ab949c73e1b742ca2fe2f7122170e7ed8ceaea9cf57c411743a5ac4c48d7405f397b63d36a25"
                ],
                "potPubkeys": [
                    "0xb60f0783433e610a3299d8c7e021f1d9201ff3945e86cdb1887b7799dde67f51dbba932100bdc504fd3c43748ef244db16c1ed2975ce432c21ce64d9795367a901468930b4e5e53501532251445de13e81be7f6c4e1381ba669c26c48f2cfdff",
                    "0x84006b5a3335426c753ed749e32c5942ef653ffede3e086d68d51476fdc3f81be58c7ebffc2866bbdfa1bef99c351a6c0e41681a14f8b6fac97e0069be1c482d14da947d2fdd02459a3e7630a7f0e4574a51c03df0792cb83cbc8a2fe792b84b",
                    "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                    "0x95b5376cc7c02b45b6f0180523ae3727f927c1def197c022ce7301697cd8f320503116a3c808bd729b854ea96fa6d2a013e255b569bf22a8fb8ee78665994e5ed73d17eb9bfbb9a36fa7ecf62742eb5f18368f801ff49956319312fa895608ec",
                    "0x88bd971ac00d2a3c47a501048f49cd9867f275a69618c4f5b8e8050578dd3776599754e0eff55ddfe7f45be90a4e56a208557f8f9baf0083b225f6229eb718a1437de56183d826e8abbf480cdf5560c82f4222c08dfa8d1061f9d6079cf624ec",
                    "0xb631d2eb6a1313c748ca9ea28a74363b23b6268a5fd5bdf3cebd502a77c5fdc0215b3c7b6652e91234d47eefc71d099b115cb0f89a1e42ec637506c949d33bfd0737e742844eb530a4df38cba7fd168ddc0ac9514e8b9dacb65c5675f0651d69"
                ],
                "blsSignatures": [""],
            }
        });

        let t = serde_json::from_value::<Transcript>(json).unwrap();

        // Verify pubkey sequence to end
        let result = verify_inclusion::<DefaultEngine>(&t, 1);
        assert_eq!(result, Ok(()));
    
    }

}