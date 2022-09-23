mod utils;

use console_error_panic_hook;
use wasm_bindgen::prelude::*;
use small_powers_of_tau::{srs::SRS, srs::Parameters, keypair::PrivateKey};

#[wasm_bindgen]
pub fn contribute(json_arr: Vec<u8>, g1_size: usize, g2_size: usize) {
    console_error_panic_hook::set_once();
    println!("Hello, wrapper-small-pot!");
    // TODO: receive entropy from JS
    let private_key = PrivateKey::rand(4);

    let parameters = Parameters {
        num_g1_elements_needed: g1_size,
        num_g2_elements_needed: g2_size,
    };
    // TODO: Vec<u8> is not json_arr type
    //let mut srs = SRS::deserialise(json_arr, parameters);
    //TODO: keep copying code from old-geoff/accumulator.rs
}
