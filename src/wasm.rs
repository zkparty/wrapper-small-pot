use std::panic;
use js_sys::Promise;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};
use wasm_bindgen_rayon::init_thread_pool;
use kzg_ceremony_crypto::{Transcript, CeremonyError};
use crate::{
    get_pot_pubkeys_with_string,
    check_subgroup_with_string,
    contribute_with_string,
    verify_with_string,
};

#[wasm_bindgen]
pub fn init_threads(n: usize) -> Promise {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_thread_pool(n)
}

#[wasm_bindgen]
pub fn contribute_wasm(input: &str, string_secret: &str, string_identity: &str) -> JsValue {
    let contribution = contribute_with_string(
        input.to_string(),
        string_secret,
        string_identity,
    ).unwrap();
    return serde_wasm_bindgen::to_value(&contribution).unwrap();
}

#[wasm_bindgen]
pub fn subgroup_check_wasm(input: &str) -> bool {
    let result = check_subgroup_with_string(input.to_string()).unwrap();
    return result;
}

#[wasm_bindgen]
pub fn get_pot_pubkeys_wasm(string_secret: &str) -> JsValue {
    let pot_pubkeys = get_pot_pubkeys_with_string(string_secret).unwrap();
    return serde_wasm_bindgen::to_value(&pot_pubkeys).unwrap();
}

#[wasm_bindgen]
pub fn verify_wasm(transcript: &str) -> bool {
    let result = verify_with_string(transcript.to_string()).unwrap();
    return result;
}

#[wasm_bindgen]
/// Verifies that a contribution is included in the transcript
pub fn verify_inclusion(t_str: string, contrib_idx: usize) -> Result<(), CeremonyError> {
    // Get transcript from json
    let t = serde_json::from_value::<Transcript>(t_str).unwrap();

    // Verify pubkey sequence to end
    t.verify_inclusion::<DefaultEngine>(contrib_idx);
}
