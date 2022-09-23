use small_powers_of_tau::{srs::SRS, srs::Parameters, keypair::PrivateKey};
use wrapper_small_pot::load_json_file;

fn main() {
    println!("Hello, wrapper-small-pot!");
    let parameters = Parameters {
        num_g1_elements_needed: 100,
        num_g2_elements_needed: 2,
    };
    let json_arr = load_json_file("initialTranscript.json");
    let srs = SRS::deserialise(json_arr, parameters);

    // TODO: receive entropy from JS
    //let private_key = PrivateKey::rand(4);

}