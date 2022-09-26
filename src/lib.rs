use eyre::Result;
use std::{fs::File, path::Path};
use ark_serialize::{Read, Write};
use small_powers_of_tau::update_proof::UpdateProof;
use small_powers_of_tau::sdk::NUM_CEREMONIES;
use small_powers_of_tau::sdk::contribution::{
    contribution_subgroup_check,
    contribution_verify_update,
    update_contribution,
    ContributionJSON,
    Contribution,
};

/**
 * We'll use this function in the cli
 */
pub fn contribute_with_file(in_path: &str, out_path: &str, proof_path: &str, string_secrets: [&str; NUM_CEREMONIES]) -> Result<()> {
    let json = read_json_file(in_path)?;
    let (result, proofs) = contribute_with_string(json, string_secrets)?;

    write_json_file(out_path, &result)?;
    write_json_file(proof_path, &proofs)
}
/**
 * We'll use this function in the wasm
 */
pub fn contribute_with_string(json: String, string_secrets: [&str; NUM_CEREMONIES]) -> Result<(String, String)> {
    let secrets = string_secrets.map(|s| s.to_string());
    let contribution = json_to_contribution(json)?;
    let (post, update_proofs) = contribute(contribution, secrets)?;

    let post_string = contribution_to_json(post)?;
    let proofs_string = update_proofs_to_json(update_proofs)?;
    Ok((post_string, proofs_string))
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
 * Util functions
 */
fn read_json_file(string_path: &str) -> Result<String> {
    let path = Path::new(string_path);
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn write_json_file(string_path: &str, content: &str) -> Result<()> {
    let buf = content.as_bytes();
    let path = Path::new(string_path);
    let mut file = File::create(path)?;
    file.write_all(buf)?;
    Ok(())
}

fn json_to_contribution(json: String) -> Result<Contribution> {
    let contribution_value = serde_json::from_str(&json).unwrap();
    let contribution_json = serde_json::from_value::<ContributionJSON>(contribution_value)?;
    let contribution = Contribution::from(&contribution_json);
    Ok(contribution)
}

fn contribution_to_json(contribution: Contribution) -> Result<String> {
    let contribution_json = ContributionJSON::from(&contribution);
    let contribution_string = serde_json::to_string(&contribution_json)?;
    Ok(contribution_string)
}

fn json_to_update_proofs(json: String) -> Result<[UpdateProof; NUM_CEREMONIES]> {
    let update_proofs_value = serde_json::from_str(&json).unwrap();
    let update_proofs_json = serde_json::from_value::<[[String; 2]; NUM_CEREMONIES]>(update_proofs_value)?;
    let update_proofs = update_proofs_json.map(|json_array| UpdateProof::deserialise(json_array).unwrap());
    Ok(update_proofs)
}


fn update_proofs_to_json(update_proofs: [UpdateProof; NUM_CEREMONIES]) -> Result<String> {
    let proofs_list = update_proofs.map(|proof: UpdateProof| proof.serialise());
    let proofs_string = serde_json::to_string(&proofs_list)?;
    Ok(proofs_string)
}
