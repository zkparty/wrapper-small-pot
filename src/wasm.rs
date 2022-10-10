use std::panic;
use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};
use wasm_bindgen_rayon::init_thread_pool;
use crate::{
    check_subgroup_with_string,
    verify_update_with_string,
    contribute_with_string,
    get_pot_pubkeys,
};

#[derive(Serialize, Deserialize)]
struct ResultTuple {
    contribution: String,
    proofs: String,
}

#[wasm_bindgen]
pub fn init_threads(n: usize) -> Promise {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_thread_pool(n)
}

#[wasm_bindgen]
pub fn contribute_wasm(input: &str, secret_0: &str, secret_1: &str, secret_2: &str, secret_3: &str) -> JsValue {
    let string_secrets = [
        secret_0,
        secret_1,
        secret_2,
        secret_3,
    ];
    let contribution = contribute_with_string(
        input.to_string(),
        string_secrets
    ).unwrap();
    return serde_wasm_bindgen::to_value(&contribution).unwrap();
}

#[wasm_bindgen]
pub fn subgroup_check_wasm(input: &str) -> bool {
    let result = check_subgroup_with_string(input.to_string()).unwrap();
    return result;
}

#[wasm_bindgen]
pub fn verify_update_wasm(input: &str, output: &str, proofs: &str, secret_0: &str, secret_1: &str, secret_2: &str, secret_3: &str) -> String {
    let string_secrets = [
        secret_0,
        secret_1,
        secret_2,
        secret_3,
    ];
    let result = verify_update_with_string(
        input.to_string(),
        output.to_string(),
        proofs.to_string(),
        string_secrets
    ).unwrap();
    return format!("{}", result);
}

#[wasm_bindgen]
pub fn get_pot_pubkeys_wasm(secret_0: &str, secret_1: &str, secret_2: &str, secret_3: &str) -> JsValue {
    let string_secrets = [
        secret_0,
        secret_1,
        secret_2,
        secret_3,
    ];
    let pot_pubkeys = get_pot_pubkeys(string_secrets).unwrap();
    return serde_wasm_bindgen::to_value(&pot_pubkeys).unwrap();
}