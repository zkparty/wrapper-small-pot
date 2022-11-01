#[cfg(target_family = "wasm")]
mod wasm;

use eyre::Result;
use hex::FromHex;
use std::str::FromStr;
use std::{fs::File, path::Path};
use ark_serialize::{Read, Write};
use small_powers_of_tau::sdk::contribution::{
    contribution_subgroup_check,
    ContributionJSON,
    Contribution,
};
use kzg_ceremony_crypto::{
    G2,
    BLST,
    Secret,
    Identity,
    get_pot_pubkeys,
    BatchContribution,
};

/**
 * We'll use this function in the cli
 */
pub fn contribute_with_file(in_path: &str, out_path: &str, string_secret: &str, string_identity: &str) -> Result<()> {
    let json = read_json_file(in_path)?;
    let contribution = contribute_with_string(json, string_secret, string_identity)?;
    let content = serde_json::to_string(&contribution)
    .expect("Result Contribution serialization failed");

    write_json_file(out_path, &content)
}
/**
 * We'll use this function in the wasm
 */
pub fn contribute_with_string(json: String, string_secret: &str, string_identity: &str) -> Result<BatchContribution> {
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

    Ok(contribution)
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
    let contribution = json_to_contribution(json)?;
    let result = check_subgroup(contribution)?;
    Ok(result)
}
/**
 * Core function: check subgroup is correct
 */
fn check_subgroup(contribution: Contribution) -> Result<bool> {
    // TODO: use the kzg-sequencer/crypto library instead of small-pot
    let result = contribution_subgroup_check(contribution);
    Ok(result)
}

/**
 * Core function: get potPubkeys from secrets
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

fn json_to_contribution(json: String) -> Result<Contribution> {
    let contribution_value = serde_json::from_str(&json)
    .expect("error parsing contribution string to json");
    let contribution_json = serde_json::from_value::<ContributionJSON>(contribution_value)
    .expect("error parsing json to contribution");
    let contribution = Contribution::from(&contribution_json);
    Ok(contribution)
}

fn string_to_entropy(string_secret: &str) -> Secret<[u8; 32]> {
    let buffer = <[u8; 32]>::from_hex(string_secret)
    .expect("secret should be a 32 hex string");
    let entropy = Secret::from(buffer);
   entropy
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
}