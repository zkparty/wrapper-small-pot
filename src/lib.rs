use eyre::Result;
use std::{fs::File, path::Path};
use ark_serialize::{Read, Write};
use small_powers_of_tau::sdk::NUM_CEREMONIES;
use small_powers_of_tau::sdk::contribution::{
    Contribution,
    ContributionJSON,
    contribution_subgroup_check,
    update_contribution
};

/**
 * We'll use this function in the cli
 */
pub fn contribute_with_file(in_path: &str, out_path: &str, string_secrets: [&str; NUM_CEREMONIES]) -> Result<()> {
    let json = read_json_file(in_path)?;
    let result = contribute_with_string(json, string_secrets)?;
    write_json_file(out_path, &result)
}
/**
 * We'll use this function in the wasm
 */
pub fn contribute_with_string(json: String, string_secrets: [&str; NUM_CEREMONIES]) -> Result<String> {
    let secrets = string_secrets.map(|s| s.to_string());

    let contribution_value = serde_json::from_str(&json).unwrap();
    let contribution_json = serde_json::from_value::<ContributionJSON>(contribution_value)?;
    let contribution = Contribution::from(&contribution_json);

    let post = contribute(contribution, secrets)?;

    let post_json = ContributionJSON::from(&post);
    let post_string = serde_json::to_string(&post_json)?;
    Ok(post_string)
}
/**
 * Core function: add participant contribution
 */
fn contribute(contribution: Contribution, secrets: [String; NUM_CEREMONIES]) -> Result<Contribution> {
    let (result, _proof) = update_contribution(
        contribution,
        secrets
    ).expect("Update contribution failed");
    Ok(result)
}


/**
 * We'll use this function in the cli
 */
pub fn check_subgroup_with_file(in_path: &str) -> Result<()> {
    let json = read_json_file(in_path)?;
    let result = check_subgroup_with_string(json)?;
    Ok(println!("user contribution is in file: {:?}", result))
}
/**
 * We'll use this function in the wasm
 */
pub fn check_subgroup_with_string(json: String) -> Result<bool> {
    let contribution_value = serde_json::from_str(&json).unwrap();
    let contribution_json = serde_json::from_value::<ContributionJSON>(contribution_value)?;
    let contribution = Contribution::from(&contribution_json);
    let result = check_subgroup(contribution)?;
    Ok(result)
}
/**
 * Core function: check participant contribution was included
 */
fn check_subgroup(contribution: Contribution) -> Result<bool> {
    let result = contribution_subgroup_check(contribution);
    Ok(result)
}
// TODO: create update_proof_check functions


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
