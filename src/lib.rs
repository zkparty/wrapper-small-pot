use eyre::Result;
use std::{fs::File, path::Path};
use ark_serialize::{Read, Write};
use small_powers_of_tau::srs::SRS;
use small_powers_of_tau::sdk::{Transcript, TranscriptJSON, transcript_subgroup_check, update_transcript};

// TEMPORARY COMMENT:
// To use functions from the project use crate::package_name
// Because library compilation is done before binary compilation
// After binary compilation (in main.rs or similar) you can use use wrapper_small_pot::package_name;

const NUM_CEREMONIES: usize = 4;

// Fix implement from in Kev's code
fn from_json(transcript_json: TranscriptJSON) -> Transcript {
    let sub_ceremonies_option: [Option<SRS>; NUM_CEREMONIES] = transcript_json
        .sub_ceremonies
        .clone()
        .map(|srs_json| (&srs_json).into());

    let mut sub_ceremonies = Vec::new();

    for optional_srs in sub_ceremonies_option {
        match optional_srs {
            Some(srs) => sub_ceremonies.push(srs),
            None => return Transcript::default(),
        }
    }

    Transcript {
        sub_ceremonies: sub_ceremonies.try_into().unwrap(),
    }
}


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
    //let seed = string_seed.to_owned();
    let secrets = string_secrets.map(|s| s.to_string());

    let transcript_value = serde_json::from_str(&json).unwrap();
    let transcript_json = serde_json::from_value::<TranscriptJSON>(transcript_value)?;
    let transcript = from_json(transcript_json);

    let post = contribute(transcript, secrets)?;

    let post_json = TranscriptJSON::from(&post);
    let post_string = serde_json::to_string(&post_json)?;
    Ok(post_string)
}
/**
 * Core function: add participant contribution to transcripts
 */
fn contribute(transcript: Transcript, secrets: [String; NUM_CEREMONIES]) -> Result<Transcript> {
    // TODO: check previous transcript
    let (result, _proof) = update_transcript(transcript, secrets).unwrap();
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
    let transcript_value = serde_json::from_str(&json).unwrap();
    // TODO: Transcript serialize is not implemented
    let transcript_json = serde_json::from_value::<TranscriptJSON>(transcript_value)?;
    let transcript = from_json(transcript_json);
    let result = check_subgroup(transcript)?;
    Ok(result)
}
/**
 * Core function: check participant contribution was included
 */
fn check_subgroup(transcript: Transcript) -> Result<bool> {
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
