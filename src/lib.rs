use eyre::Result;
use ark_serialize::{Read, Write};
use std::{fs::File, path::Path};
use small_powers_of_tau::sdk::{Transcript, transcript_subgroup_check, update_transcript};

// TEMPORARY COMMENT:
// To use functions from the project use crate::package_name
// Because library compilation is done before binary compilation
// After binary compilation (in main.rs or similar) you can use use wrapper_small_pot::package_name;

/**
 * We'll use this function in the cli
 */
pub fn contribute_with_file(in_path: &str, out_path: &str, string_seed: &str) -> Result<()> {
    let json = read_json_file(in_path)?;
    let result = contribute_with_string(json, string_seed)?;
    write_json_file(out_path, &result)
}
/**
 * We'll use this function in the wasm
 */
pub fn contribute_with_string(json: String, string_seed: &str) -> Result<String> {
    let seed = string_seed.to_owned();
    let previous_json = serde_json::from_str(&json).unwrap();
    // TODO: Transcript serialize is not implemented
    let previous = serde_json::from_value::<Transcript>(previous_json)?;
    let post = contribute(previous, seed)?;
    let post_string = serde_json::to_string(&post)?;
    Ok(post_string)
}
/**
 * Core function: add participant contribution to transcripts
 */
fn contribute(previous: Transcript, seed: String) -> Result<Transcript> {
    // TODO: create secrets
    let result = update_transcript(transcript, secrets)?;
    // TODO: check previous
    // TODO: perform update
    Ok(Transcript::default())
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
    let json = serde_json::from_str(&json).unwrap();
    // TODO: Transcript serialize is not implemented
    let transcript = serde_json::from_value::<Transcript>(json)?;
    let result = subgroup_check(transcript)?;
    Ok(result)
}
/**
 * Core function: check participant contribution was included
 */
fn subgroup_check(transcript: Transcript) -> Result<bool> {
    let result = transcript_subgroup_check(transcript);
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
