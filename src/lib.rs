#[cfg(target_family = "wasm")]
mod wasm;

use eyre::Result;
use hex::FromHex;
use std::{fs::File, path::Path};
use ark_serialize::{Read, Write};
use ark_ec::{ProjectiveCurve};
use small_powers_of_tau::keypair::PrivateKey;
use small_powers_of_tau::update_proof::UpdateProof;
use small_powers_of_tau::interop_point_encoding::serialize_g2;
use small_powers_of_tau::sdk::NUM_CEREMONIES;
use small_powers_of_tau::sdk::contribution::{
    contribution_subgroup_check,
    contribution_verify_update,
    update_contribution,
    ContributionJSON,
    Contribution,
};
use kzg_ceremony_crypto::{BatchContribution, BLST, derive_taus};

/**
 * We'll use this function in the cli
 */
pub fn contribute_with_file(in_path: &str, out_path: &str, proof_path: &str, string_secrets: [&str; NUM_CEREMONIES]) -> Result<()> {
    let json = read_json_file(in_path)?;
    let result = contribute_with_string(json, string_secrets)?;

    write_json_file(out_path, &result)
}
/**
 * We'll use this function in the wasm
 */
pub fn contribute_with_string(json: String, string_secrets: [&str; NUM_CEREMONIES]) -> Result<String> {
    let secrets = string_secrets.map(|s| s.to_string());

    let mut contribution = serde_json::from_str::<BatchContribution>(&json)?;
    contribution.add_entropy::<BLST>(&secrets.into())?;
    Ok(serde_json::to_string(&contribution)?)

    /*
    let contribution = json_to_contribution(json)?;
    let (post, update_proofs) = contribute(contribution, secrets)?;

    let mut post_json = ContributionJSON::from(&post);
    // you can also get potPubekys from updateProofJSON[0] values
    let pot_pubkeys = get_pot_pubkeys(string_secrets)
    .expect("error getting pot pubkeys from secrets");
    for (i, pot_pubkey) in pot_pubkeys.into_iter().enumerate() {
        post_json.contributions[i].pot_pubkey = pot_pubkey;
    }
    let post_string = serde_json::to_string(&post_json)
    .expect("error serializing contribution json to string");

    let proofs_string = update_proofs_to_json(update_proofs)?;
    Ok((post_string, proofs_string))
    */

}
/**
 * Core function: add participant contribution
 */
fn contribute(contribution: Contribution, secrets: [String; NUM_CEREMONIES]) -> Result<(Contribution, [UpdateProof; NUM_CEREMONIES])> {
    let (result, proofs) = update_contribution(
        contribution,
        secrets
    ).expect("Update contribution failed");
    Ok((result, proofs))
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
    let result = contribution_subgroup_check(contribution);
    Ok(result)
}


/**
 * We'll use this function in the cli
 */
pub fn verify_update_with_file(in_path: &str, out_path: &str, proof_path: &str, string_secrets: [&str; NUM_CEREMONIES]) -> Result<()> {
    let old_json = read_json_file(in_path)?;
    let new_json = read_json_file(out_path)?;
    let proof_json = read_json_file(proof_path)?;
    let result = verify_update_with_string(old_json, new_json, proof_json, string_secrets)?;
    Ok(println!("Contribution was included: {:?}", result))
}
/**
 * We'll use this function in the wasm
 */
pub fn verify_update_with_string(old_json: String, new_json: String, proof_json: String, string_secrets: [&str; NUM_CEREMONIES]) -> Result<bool> {
    let old_contribution = json_to_contribution(old_json)?;
    let new_contribution = json_to_contribution(new_json)?;
    let update_proofs = json_to_update_proofs(proof_json)?;
    let secrets = string_secrets.map(|s| s.to_string());

    let result = verify_update(old_contribution, new_contribution, secrets, update_proofs)?;
    Ok(result)
}
/**
 * Core function: check contribution was included using update proof
 */
fn verify_update(old_contribution: Contribution, new_contribution: Contribution, secrets: [String; NUM_CEREMONIES], update_proofs: [UpdateProof; NUM_CEREMONIES]) -> Result<bool> {
    let result = contribution_verify_update(
        &old_contribution,
        &new_contribution,
        &update_proofs,
        secrets,
    );
    Ok(result)
}

/**
 * Core function: get potPubkeys from secrets
 */
pub fn get_pot_pubkeys(string_secrets: [&str; NUM_CEREMONIES]) -> Result<Vec<String>> {
    let secrets = string_secrets.map(|s| s.to_string());

    let taus = derive_taus::<BLST>(&secrets.into(), NUM_CEREMONIES);
    Ok(taus)

    /*
    let mut pot_pubkeys = Vec::with_capacity(NUM_CEREMONIES);
    for(_i, secret_string) in string_secrets.into_iter().enumerate() {
        let secret_hex = secret_string.to_string();
        if let Some(secret_stripped) = secret_hex.strip_prefix("0x") {
            let bytes = <[u8; 32]>::from_hex(secret_stripped)
            .expect("secret is not 64 characters long");
            if !bytes.is_empty() {
                let private_key = PrivateKey::from_bytes(&bytes);
                let mut a = hex::encode(serialize_g2(&private_key.to_public().into_affine()));
                a.insert_str(0, "0x");
                pot_pubkeys.push(a);
            }
        }
    }
    Ok(pot_pubkeys)
    */
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

fn json_to_update_proofs(json: String) -> Result<[UpdateProof; NUM_CEREMONIES]> {
    let update_proofs_value = serde_json::from_str(&json)
    .expect("error parsing update proof string to json");
    let update_proofs_json = serde_json::from_value::<[[String; 2]; NUM_CEREMONIES]>(update_proofs_value)
    .expect("error parsing json to update proofs object");
    let update_proofs = update_proofs_json.map(|json_array| UpdateProof::deserialise(json_array)
    .expect("error parsing json to specific update proof"));
    Ok(update_proofs)
}

fn update_proofs_to_json(update_proofs: [UpdateProof; NUM_CEREMONIES]) -> Result<String> {
    let proofs_list = update_proofs.map(|proof: UpdateProof| proof.serialise());
    let proofs_string = serde_json::to_string(&proofs_list)
    .expect("error serializing update proofs to string");
    Ok(proofs_string)
}
