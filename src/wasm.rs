use std::panic;
use js_sys::Promise;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};
use wasm_bindgen_rayon::init_thread_pool;
use kzg_ceremony_crypto::{Transcript, CeremonyError};
use crate::{
    get_pot_pubkeys_with_string,
    check_subgroup_with_string,
    contribute_with_string,
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
/// Verifies that a contribution is included in the transcript
pub fn verify_inclusion(t_str: string, contrib_idx: usize) -> Result<(), CeremonyError> {
    // Get transcript from json
    let t = serde_json::from_value::<Transcript>(t_str).unwrap();

    // Verify pubkey sequence to end
    t.verify_inclusion::<DefaultEngine>(contrib_idx);
}

#[cfg(test)]
mod test {
    use kzg_ceremony_crypto::{
        CeremonyError,
        DefaultEngine,
    };

    #[test]
    fn test_verify_inclusion() {
        let json = r###"{
            "numG1Powers": 4,
            "numG2Powers": 2,
            "powersOfTau": {
                "G1Powers": [
                    "0x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                    "0x962dffcca938bfe9cc11ab949c73e1b742ca2fe2f7122170e7ed8ceaea9cf57c411743a5ac4c48d7405f397b63d36a25",
                    "0xa57913e7354d2bdbb631e7b270ad9b0fd34c8ee177c5f0903024cc1da1221fa65c92ba515473aa248137dfc510d5d4c9",
                    "0xb2581616a7420d485eee433d355540fd2d9f441a7864b168ad0e068abf4772fa2f644a192b3953616f9fbd5ac88dcd64",

                ],
                "G2Powers": [
                    "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                    "0x8bab27ba31974bfd253d0d37e8ec7c580fa5cc9dfd81eb2e6faae3c4b38b3efcf1e9f3513a0b4031662bbabefe656614190d96e2503d3c68d44324722f00d3abb1b4ec0ba7aa1f4a8595487649912f87ea8f6761648425e9b9baa5a46e18f2c9",
                ],
            },
            "witness": {
                "runningProducts": [
                    "0x9938667a3807b2bba879c10272b0470507cb5926784976d34a440f6ef1aa0cad8a6963c790642a5e76e67cb6471ee075",
                    "0x94d3d149a60414d74e1e60fd1bb4d08ffef7860d86e698439ebbea8614d7ece57d32a1f9be83389bdf7c64846af2513d",
                    "0x94d3d149a60414d74e1e60fd1bb4d08ffef7860d86e698439ebbea8614d7ece57d32a1f9be83389bdf7c64846af2513d",
                    "0xa6ed552088642b976df969e47dc6503e33450744de1b358cafdd610dda3edbfc92efc98c77a4960010a89fa60d0dd127",
                    "0xb4d01655876e28edfa71521ab9fc5d916d9f3ea1c51477c7f912501246a9a7643d2f4ae971563e98dfbdd28df764bedd",
                    "0x962dffcca938bfe9cc11ab949c73e1b742ca2fe2f7122170e7ed8ceaea9cf57c411743a5ac4c48d7405f397b63d36a25"
                ],
                "potPubkeys": [
                    "0xb60f0783433e610a3299d8c7e021f1d9201ff3945e86cdb1887b7799dde67f51dbba932100bdc504fd3c43748ef244db16c1ed2975ce432c21ce64d9795367a901468930b4e5e53501532251445de13e81be7f6c4e1381ba669c26c48f2cfdff",
                    "0x84006b5a3335426c753ed749e32c5942ef653ffede3e086d68d51476fdc3f81be58c7ebffc2866bbdfa1bef99c351a6c0e41681a14f8b6fac97e0069be1c482d14da947d2fdd02459a3e7630a7f0e4574a51c03df0792cb83cbc8a2fe792b84b",
                    "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                    "0x95b5376cc7c02b45b6f0180523ae3727f927c1def197c022ce7301697cd8f320503116a3c808bd729b854ea96fa6d2a013e255b569bf22a8fb8ee78665994e5ed73d17eb9bfbb9a36fa7ecf62742eb5f18368f801ff49956319312fa895608ec",
                    "0x88bd971ac00d2a3c47a501048f49cd9867f275a69618c4f5b8e8050578dd3776599754e0eff55ddfe7f45be90a4e56a208557f8f9baf0083b225f6229eb718a1437de56183d826e8abbf480cdf5560c82f4222c08dfa8d1061f9d6079cf624ec",
                    "0xb631d2eb6a1313c748ca9ea28a74363b23b6268a5fd5bdf3cebd502a77c5fdc0215b3c7b6652e91234d47eefc71d099b115cb0f89a1e42ec637506c949d33bfd0737e742844eb530a4df38cba7fd168ddc0ac9514e8b9dacb65c5675f0651d69"
                ],
                "blsSignatures": [""],
            }
        }"###;
        let result = verify_inclusion(json, 1);
        assert_eq!(result, Ok(()));
    }
}
