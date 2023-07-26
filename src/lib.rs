#[cfg(target_family = "wasm")]
mod wasm;

use eyre::Result;
use hex::FromHex;
use std::str::FromStr;
use std::{fs::File, path::Path};
use ark_serialize::{Read, Write};
//use serde::{Deserialize, Serialize};
use kzg_ceremony_crypto::{
    G2,
    BLST,
    Secret,
    Identity,
    get_pot_pubkeys,
    BatchContribution,
    BatchTranscript,
    Transcript,
    CeremonyError,
    Engine,
};
use rayon::prelude::*;

/**
 * We'll use this function in the cli
 */
pub fn contribute_with_file(in_path: &str, out_path: &str, string_secret: &str, string_identity: &str) -> Result<()> {
    let json = read_json_file(in_path)?;
    let contribution = contribute_with_string(json, string_secret, string_identity)?;

    write_json_file(out_path, &contribution)
}
/**
 * We'll use this function in the wasm
 */
pub fn contribute_with_string(json: String, string_secret: &str, string_identity: &str) -> Result<String> {
    // parse contribution object
    let mut contribution = serde_json::from_str::<BatchContribution>(&json)
    .expect("Contribution deserialization failed");
    // parse entropy
    let entropy = string_to_entropy(string_secret);
    // parse identity (eth or git)
    let identity = Identity::from_str(string_identity)
    .expect("Identity deserialization failed");

    contribution.add_entropy::<BLST>(&entropy, &identity)
    .expect("Contribution computation failed");
    let result = serde_json::to_string(&contribution)
    .expect("Result Contribution serialization failed");

    Ok(result)
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
    // parse contribution object
    let mut contribution = serde_json::from_str::<BatchContribution>(&json)
    .expect("Contribution deserialization failed");

    let result = contribution.validate::<BLST>();

    let is_valid = match result {
        Ok(()) => true,
        Err(error) => {
            println!("{:?}", error);
            false
        },
    };
    Ok(is_valid)
}
/**
 * We'll use this function in the wasm
 */
pub fn get_pot_pubkeys_with_string(string_secret: &str) -> Result<Vec<G2>> {
    let entropy = string_to_entropy(string_secret);
    let pot_pubkeys = get_pot_pubkeys::<BLST>(&entropy);
    Ok(pot_pubkeys)
}
/**
 * We'll use this function in the cli
 */
pub fn verify_with_file(in_path: &str) -> Result<()> {
    let json = read_json_file(in_path)?;
    let result = verify_with_string(json)?;
    Ok(println!("Verification is correct: {:?}", result))
}
/**
 * We'll use this function in the wasm
 */
pub fn verify_with_string(json: String) -> Result<bool> {
    // parse batch transcript object
    let batch_transcript = serde_json::from_str::<BatchTranscript>(&json)
    .expect("BatchTranscript deserialization failed");

    let sizes = vec![(4096, 65)];
    let result = batch_transcript.verify_self::<BLST>(sizes);

    let is_valid = match result {
        Ok(()) => true,
        Err(error) => {
            println!("{:?}", error);
            false
        },
    };
    Ok(is_valid)
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

fn string_to_entropy(string_secret: &str) -> Secret<[u8; 32]> {
    let buffer = <[u8; 32]>::from_hex(string_secret)
    .expect("secret should be a 32 hex string");
    let entropy = Secret::from(buffer);
   entropy
}

/// Verifies that a contribution is included in the transcript
fn verify_inclusion<E: Engine>(t: &Transcript, contrib_idx: usize) -> Result<(), CeremonyError> {
    assert!(contrib_idx < t.witness.products.len());

    // Loop through subsequent witness entries. Do pairing check on each.

    if t
        .witness
        .pubkeys
        .par_iter()
        .enumerate()
        .filter(| (i, _) | i>=&contrib_idx)
        .map(| (_, p) | p)
        .any(| pubkey | *pubkey == G2::zero())
            { return Err(CeremonyError::ZeroPubkey); };

    if t
        .witness
        .products
        .par_iter()
        .enumerate()
        .filter(| (i, _) | i>=&contrib_idx)
        .any(|(i, product)|
            // Pairing check: this & prev products, this pubkey
            E::verify_pubkey(
                *product,
                t.witness.products[i - 1],
                t.witness.pubkeys[i],
            ).is_err())
            {
                return Err(CeremonyError::PubKeyPairingFailed)
            };

    Ok(())
}

// Validate all transcripts for a given id
fn verify_with_id<E:Engine>(bt: &BatchTranscript, id: Identity) -> Result<(), CeremonyError> {
    let index = bt
        .participant_ids
        .par_iter()
        .position(| u | *u == id)
        .unwrap();

    if bt
        .transcripts
        .iter()
        .any( | t | verify_inclusion::<E>(t, index).is_err()) {
            return Err(CeremonyError::PubKeyPairingFailed);
        };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kzg_ceremony_crypto::DefaultEngine;

    #[test]
    fn secrets_to_pubkey_test() {
        // This test ensures that pubkeys dericvation appears correct
        let string_secret = "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b";

        let pot_pubkeys = get_pot_pubkeys_with_string(string_secret).unwrap();
        println!("{:?}", serde_json::to_value(&pot_pubkeys[0]).unwrap());
        println!("{:?}", serde_json::to_value(&pot_pubkeys[1]).unwrap());
        println!("{:?}", serde_json::to_value(&pot_pubkeys[2]).unwrap());
        println!("{:?}", serde_json::to_value(&pot_pubkeys[3]).unwrap());

        let value_pot_pubkeys_0 = serde_json::to_string(&pot_pubkeys[0]).unwrap();
        assert_eq!(value_pot_pubkeys_0.get(0..3).unwrap(), String::from("\"0x"));
        assert_eq!(value_pot_pubkeys_0.len(), 194 + 2);
    }

    #[test]
    fn test_verify_inclusion() {
        let json = serde_json::json!({
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
        });

        let mut t = serde_json::from_value::<Transcript>(json).unwrap();

        // Verify pubkey sequence to end
        let result = verify_inclusion::<DefaultEngine>(&t, 1);
        assert_eq!(Ok(()), result);

        // Make it fail with a 0 point
        t.witness.pubkeys[5] = G2::zero();
        let result2 = verify_inclusion::<DefaultEngine>(&t, 1);
        assert_eq!(Err(CeremonyError::ZeroPubkey), result2);

    }


    #[test]
    fn test_verify_with_id() {
        let json = serde_json::json!({
            "transcripts": [
              {
                "numG1Powers": 7,
                "numG2Powers": 5,
                "powersOfTau": {
                  "G1Powers": [
                    "0x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                    "0x962dffcca938bfe9cc11ab949c73e1b742ca2fe2f7122170e7ed8ceaea9cf57c411743a5ac4c48d7405f397b63d36a25",
                    "0xa57913e7354d2bdbb631e7b270ad9b0fd34c8ee177c5f0903024cc1da1221fa65c92ba515473aa248137dfc510d5d4c9",
                    "0xb2581616a7420d485eee433d355540fd2d9f441a7864b168ad0e068abf4772fa2f644a192b3953616f9fbd5ac88dcd64",
                    "0xb67698857fe9b811c820d2ae3e2e5267f31526abe192b251ec3935e31143ca3da23de6a8e42721f2589bd56a0a8e8140",
                    "0x982fb712a2e943fed5813e332b281c9a80020cb8f7d99bf0f64b7ba76b459e0f1d888ff192e53974f1af8db3ddf22932",
                    "0xa8de6a08f83307e857abdc0a32975f617f3867b7a226b12b64d2150c22c83c92aa7aa7eab77d4f4aa5285dd96723534f",
                  ],
                  "G2Powers": [
                    "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                    "0x8bab27ba31974bfd253d0d37e8ec7c580fa5cc9dfd81eb2e6faae3c4b38b3efcf1e9f3513a0b4031662bbabefe656614190d96e2503d3c68d44324722f00d3abb1b4ec0ba7aa1f4a8595487649912f87ea8f6761648425e9b9baa5a46e18f2c9",
                    "0x85206b3739624ab96b18e1c6aed5c41f5c5f37202787e34a19d9849f22b634cc4c56b669d2305c34f23ea56dba3cf5401175a7ce47e8306809db52f1a6a0be27d242b123ae3976597aa6f870a3b8b540c386a96781f459daf989861ea70205bc",
                    "0x87f2520c73d22d58b9d095e6a236a21e0f221676143f41a41846fefe879ffcac07d835395927bd2163f1b7934593298d15320c491ba04cdc52bcd500ce094e58cc2bfa7bb2a4c5a8b399aa8e6ce4da86abe806fc0c2be71be014124ecfe8101c",
                    "0x83233be258e4905e7b20712b53e59a1f14cc1c00a76755f2578d24cdfa41ea2472d2770760d8d7a331f3cd22872cf2fe18d84ae9ab6216a0c9ece4015ba06ae1f5352e6804be9219c6ebe5c4f75973d5211392975eff3135fd5ccba53b605c4a",
                  ]
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
                 "blsSignatures": [
                    "0x864195aef19c415ea3841d446cfc9ba75660f6dea2af737a78632d8a2f5aed85ce732b34e24eaa44a22e4d7b62c3f6d2",
                    "",
                    "",
                    "0x982d7da6dcae037c24839355d397014fddcaf8d08f5a699d755e3214ba7a76f67673a4c47df1fd1f85ed5a0ce619d247",
                    "0x995fd005a925310f1af55622a826e4778c258199d4dc6edf9afe2cacc9e7d93bfdd7b73edd705820aa65220f4c245ea8",
                    "0xb90c039a1edfcdbd25cb425f6452b0ca06a7be3e23d9de00f065f7849c5814c3e12e940a0fbbbac7710d6379f4a4a965"
                  ]
                }
              },
              {
                "numG1Powers": 5,
                "numG2Powers": 5,
                "powersOfTau": {
                  "G1Powers": [
                    "0x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                    "0xb10f900f90447a2c27e78a421d390d6b6921bb625d047a079163a4286698686175070f879d561e41c10a6d6ef9ed47b2",
                    "0xb92ef074a6631067bb7b847bc9047ba39b8517b1d581d3d8b1e7c68e08bfc5c1fb458b1b5cf4a92b9eb240f89a96e9ab",
                    "0xaee3584aaea093d925d3ce8ba338a9b5e3ea918eb7618fb708b79c68707530e650e6e6a47e3293cd7ef98ce62addae24",
                    "0x87e07399817e27718be8a31a9f902fe6e31e6bf59120bfa6ca58b2040e74fcd5224b6745bbbfbf35acaaed802aa3e219",
                  ],
                  "G2Powers": [
                    "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                    "0xa02f6d5ac0192e188d95eb9af56909c8a483bc8591c32b23653523855dfcf556b7d31f4fd54da0d447e24a56a519bcfe01f9372a72a1ffa64964f77c3eca1ccd4084aa364e71c8a93a4515474d26311082abec09428a017229319d3b73c18b87",
                    "0x931c15ae17f60cc675a8434bf7a6b1e42434a496fefbe035b69103c662db412b02cbaeef7a85de6cf84811341c3839340efd7a2f2005411c94dfa2caff4ba341f43d44f6f3a1b0739bf6b3a1266980f832dbd5fa9f479f6b416a0572aae79b61",
                    "0x94874106ed9cf0733f03e2734e092216ee7d751be8d55d01c7277c45da32ae7008469d6bd774703167fb2c49cbcd6dca02e311f56a1a270c66dd54fb49a62c0b6ecb773a5af2a465acfb59246220ea5ddfab5916ba474b6605985f05e4d2e0d9",
                    "0xa48cc5ec540a6e29108faee1a77adb9f14deb0dcdec8b459f74cdc4e697569e8c083e8dddba0edff60f117b8aa64e831159af960ce8c11fcda044609eaa6c96392121d40db9dd651b58085838fc454427f3ff1e42da34bfbc1dcd9def4287a8b",
                  ]
                },
                "witness": {
                  "runningProducts": [
                    "0xab39bce967908839aac5e71c4e76a77ae6112cecdf4a954e05058cae9e4afc1d69476991c172d1afebc6f6ea0b49ebcc",
                    "0x979a6835b3d215e53eb6a9de844299a5fd0672afd8336f988135c92efb0c2c9e98a4637b464d4c865ce1884ae14773e2",
                    "0x979a6835b3d215e53eb6a9de844299a5fd0672afd8336f988135c92efb0c2c9e98a4637b464d4c865ce1884ae14773e2",
                    "0x8e61b40faa315fdd9c9525aa54d9189614a8b60697a720cc0237309062df37c31e13f96431ae510c6881fae35968821b",
                    "0xa27c13ccfbfe344fadb7ce1b5ce045fa49b02feb4df9c9382fc18a674e2972ec9837340cbfc195766dfeb1ffcf983a33",
                    "0xb10f900f90447a2c27e78a421d390d6b6921bb625d047a079163a4286698686175070f879d561e41c10a6d6ef9ed47b2"
                  ],
                  "potPubkeys": [
                    "0x8705d24df98de27dc342c8d22e779cfaea5973298e1bc59d35501da3ff5c0ba61b36a4f8c244a7cc8ac124cfd96b4f950f9c00b0268bc58c73b31fa4d167f221050b768b6847732d39eb6adba94afa53d50363b51d07ae0928b23f9a3173a6ec",
                    "0xb080907be82a0313f161924a8e72d83d8a05fd68d2012ece20328990d4b1a6941b22389a68f79ef9f06e416b25c62f6917898d2705b2feab16fad85cd690679ad3ddeb6178a6d69c0c85f20c199cbd3b4ea607f38f1c378be00fb56ba65416c8",
                    "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                    "0x8d08f181174e5dfcbe8f406003a4273380a586dc225924b706494c3488baba4f44e2949e0f085b971a7d6dcc7122c29f1552de5fb69ca40272a351885815c518f40d72e0d3a0fe88fee4ca33dff42854a79d81219acaf389c9eaa894576216ed",
                    "0x805903ae522e518bd9d8167d408ac28f657611892b4dee6aa9a9d76c905ce1da150732fba869ce302673d5dbfaf0bfec1019b5df8b330d06143be51f9cac0c2af365fe97d0547f6e91f5b04fa2b8f7c51bfce42bc5db62067a55b7cd88837b90",
                    "0x875f126d77434bbd22ad644e37579db9809b4500b0f3399ca7a3ee485fc285d69276def7e589e982a5ec0e00613798d515bf9dacb777eb091482ef3f9ca80988c53d163f34356c65360fdba0ca98d76bcd265e63aebbbecae9009102b8a6e41b"
                  ],
                  "blsSignatures": [
                    "0xb6b1a6711e0e7a4a04807a307320186c9a30b2b36d1892aaf6307035126ad5c1899470ec25422caa32d27b0f51070efb",
                    "",
                    "",
                    "0xa522d782de6a68afa8eaa572e16895d8ba6efb599e392ae7aef3097162e99b6351e247cdaa2167fd84b6c4dae0fa048e",
                    "0x8f39f5f54e346cc440518d0ba3c4b2bc4816a2e34accc0347cfa999c3cfc4bcae7f5acf5ea7b31b8fd891c941e89752e",
                    "0xa763434bbfb397e6f72aef92ffec130b28d38ac66c6918e0e987cb3d579c8e985bca4a79201038e8edc4fed2c76663c9"
                  ]
                }
              },
              {
                "numG1Powers": 4,
                "numG2Powers": 5,
                "powersOfTau": {
                  "G1Powers": [
                    "0x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                    "0x81c51d85522684f50c8ca77a88a98f82eef2e2d81d4aeb8003363871f916fbdfc54e1fb010cd0d5db971b93b2a1b21eb",
                    "0xae035a3c12420a5f9e37685400457e82891275251a784a1d47bffda89cbb59879993dab6de4664499d7ea4e1fc3e1884",
                    "0xb629e784d085280a731a8d135dc79b34a923f1e1c4111cc5189ab48f5c3797b5711ddb189976b7706b3590cf7894ed3a",
                   ],
                  "G2Powers": [
                    "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                    "0x8f3cc601d7031e0cc252f790e79f7431b8b38e6e068c28fdd7aeeb519d415a90b12d5a8849fa903825c82c4455110b560d3984da83414add3e372dc3055587f4da1e733ab67758c7b88b26a855c475a0c4241067030eb1fd06846996d9e956fb",
                    "0x842780f0ab6b0d0b19684b92d12e0239c232e60f7ec8451f5b1bddbaf1633d43686ef9a1bcb5c128aa6ceae61360563f08ce8d2260ff1df5fcfaba78b898522175e19706f27b06fef7cc492cc6ae676432bd214548d4a6db5a8b9351556256e7",
                    "0xa25bc3dea9457d8ac0a3f11e2f8a4a8bfe34befa02f157981edce4e328c1d2afb3a4ada8d72fff33a1d9fce0dea916ad1882f64919344aff8311be61cb94a8aa9dcb8f574666871bf7a8c028d72a7d7c28ff0373bead323bae5597fda964eb1c",
                    "0x8d0729a64b73a90a33c9c15f54c84d0705621cc00a1bb8eef2cb62c0c6fc6b2dee463b7041b802e55f032aef95b5ca2a18df539f2ba0976063db287e59ad61c3939a2a2ab595e4b9dc4a3ab42448f413c740ec75f5a45e7aaf1196eddeb97ae0",
                  ]
                },
                "witness": {
                  "runningProducts": [
                    "0x8a6209aa6f5d02f1ac6e6e4aa52adab697218fb005df3b9e1d731b7d162821954c6f14e69e354d80567093ebf40a4d1e",
                    "0xad04b658f79a05aaad05b6d31af098932407e1543fef1a6a118bda4a1fea417f1add8ca022f8f54f4006ff0e07e06981",
                    "0xad04b658f79a05aaad05b6d31af098932407e1543fef1a6a118bda4a1fea417f1add8ca022f8f54f4006ff0e07e06981",
                    "0xa306239f0caebfb2b6ea8a69eb9bd6ab88429c8e93617f772eb966461749ff1a73a0e593856baf0f15db654cc6231cf8",
                    "0xaa7bca3423c330288616aea6a5f454b3d72bbfbdba8bb6fc24c9c45503ba1bf5d096135358d0e182b7bd8fcccb1e087e",
                    "0x81c51d85522684f50c8ca77a88a98f82eef2e2d81d4aeb8003363871f916fbdfc54e1fb010cd0d5db971b93b2a1b21eb"
                  ],
                  "potPubkeys": [
                    "0xb9a2e47063ecd62734dfcff179e38fa540bf0c8bcf57cbed0aa36afccaba84a8e53368f68cce6b0714793130c8be1daf120d776790a583ff0baf241609e491d9a9b740b32e5c0ba18b82b7a14745f7d555c49f2edde4ebff04dac7df843e3c02",
                    "0x95c0abf7600f14bac4b1956757b4c4592619cdfb309b2db7c32a99820479c83be1ce282bf4297c15623f6adedce7615c17d523affba425891db4ee9627a12b4947be5590e6edc01ae9aaad72af81faca31bce8f11354222de91d5a9547fb5885",
                    "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                    "0xa1616d868079d5221e8e73771c0d5168c4c460e408ab0d8aa2b8f7a9130b244deddac19f890779a0069879e340df398e048c47efb22507e3bb8fcb2fd15fbce28e11715d7731207cd989d4989e8e802906f79b20215cbc122674d7a5e0fbbf9c",
                    "0x99b536c487dc7d44dc2bb90565083d9822fdd850219f0d1c95793dbee11e37b96f9e8f996bfd3d2c11eb33024c81c443186003623ffb9f839355ae9ae6f64e932c6b7bd878fbc53e60d8bbda08a8272712b9934cb43cd5dac052a952e8356cc2",
                    "0xb349b925664606449f6d841ff35243dd52dab6e74e505aecff8e3b0152235d0d50e320c5b684a67764124682691031b50bb8bf5cb62b5927efb789a768d2020972952fc7e7cde7060e74961bdf4c9ae3fd0dc602ac445553f650d3ad89a8cf31"
                  ],
                  "blsSignatures": [
                     "0x8dcbee5e31c5fdb15be64d62b7b74dd39e33bc2650e78d14dae76f41fb4fa8ee8c16d56145320565c2fe614d6d0df806",
                    "",
                    "",
                    "0x8214103137bdff8e2642ecde0bec7f73fbe99fa8a1c779eaf53accac2708dc58003a9cdb54ac313e0cfdc36319d22fd3",
                    "0x91066f3099209bd91e135607451e8045503235452ddcd4d6318ae1f95b3e01cd36bf1ed03b75bd64966923f2092f9959",
                    "0x8c67401bf2d4de5c99064c60ca8b53ecd419f443429fdf74244a6c3bf45315031b1cfc066b2765ed7c5ee68cbfd60e67"
                  ]
                }
              },
              {
                "numG1Powers": 6,
                "numG2Powers": 5,
                "powersOfTau": {
                  "G1Powers": [
                    "0x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                    "0xa20210122637e2a91c14e76bbd07f27c2fac87030f67bc2ed51d093bea7bca5b33ad4c9359c806c41fd3aea50fc5edc8",
                    "0x8afe6b5d89d2a956399f6a4d790ffaee67dd0709904119657add0f6a5b2d578d136ebd6b1ea3f65699e04a2acbb19a9d",
                    "0xadd258b68684c764ac4db7731425b78561d019d456a2e2dfa25df6258fcbc25891aef20465d965edcbdb5370b6d43b31",
                    "0x8c151700274b8be3f38e9e0704208c9b7a2c58dd69075d6a800f960c2abdf3dbcc9eaf7627be4d732ec71c4e62358324",
                    "0xa1b8cfde67818f9b831f8d3b416e198e41c524b3c913738a4ad3e9e90fb00386e4b204a0f87cb47f65453c9c14a65deb",
                  ],
                  "G2Powers": [
                    "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                    "0x8ab31defe056457f459bfc12e06afbfde7a0536b4747011ba9d1c2d4c4b5012b070b0ee967676c4154dbcfd85206dcec021676f54a44a40f3e65fe2682e0908a082514aa05da6872a615523f2e7e587ea6afdbd42a979b68cab98507bf0a48ab",
                    "0x80f522470b3bb5865fe3c9fbe58f521fe275886429136ab28d305308ba4aed1ca47440a4ec7d51511855a574765ae60a09f484dc36b3a3e0796e525133dcb008145578da8ba8d7296b448f7390ab22bf1face28e9a9b1b75c159b3752331729d",
                    "0x9810d454244de7fd7a8addf0a2ad608ef53bb244849aff4eb8e8c72865035c8579fb4ee80b1db7ddee066e2401237a8b0dab1b6650b2c9d8cd844569e3c4b538a45116fb3d36111bb481e723495ef3062ce5117bcf508e325b00bf2d35736e35",
                    "0xb735c4f7b0c8c664cabf1d1ea1c088afe2df13f3d1e6b96a144df3ff53f64ca025770cde7bf3eb0e3080d011d0c579ed171c8db2996028f9ab9c5eea83658b863046586ebc01bd4edcc24d4a4305cd9ca592609165a000a0c568a4bf043eb2fd",
                  ]
                },
                "witness": {
                  "runningProducts": [
                    "0xa8604f6cf76b8d4c0ba5a069d8a042e1ff593761e0b92f4b3d11295245014afebfb5e158cd52a398877ef47615048a3b",
                    "0x8329b6ccc72202cad654fbba942c3621c5382ac8c1db18d4a12328cd532dda9e5bfa0f83f8ffbf3cff592c03244e1136",
                    "0x8329b6ccc72202cad654fbba942c3621c5382ac8c1db18d4a12328cd532dda9e5bfa0f83f8ffbf3cff592c03244e1136",
                    "0xac283e119ef58387700de895ef50700e8811838339fb8e54b4da2508b57c4155ebbb4883945d9099cf5d5b83dc5ade08",
                    "0xa9a8b66a153b53095b30761f650a851a9493aa510015d88b5e5d99b6f77176730eb1eaf41deca827509a7afbf2a394e5",
                    "0xa20210122637e2a91c14e76bbd07f27c2fac87030f67bc2ed51d093bea7bca5b33ad4c9359c806c41fd3aea50fc5edc8"
                  ],
                  "potPubkeys": [
                    "0xa8b550a52a4512793a2d248daa9a590d8d94da6546244cd4535f1e53868a37eef9c01c75dccfc1ce13e0d5bcbb92528811a7a5320152822a9b82a9803ed4dfe28aacf32d9e911fb39a59a7da09809f4f18d92e75485dcea5f2fcb353cb864cb2",
                    "0xa5da9b870afcdcdb9e09130a256716a82514f73e1a7b752581274905b9d18943c599ec60d3e9be017c3191adae4737c8130c4c824315eb2075975808df015d5d93c2ac5d3636b046c4ee295d47be275f6c13bfb43ca9ed917accd76ba0a2c068",
                    "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                    "0x959f949222f8d10d937789fed3e623675552fafbdee7efccef42ef74b90ac1ffa073562d7bd848b156b5e207cb900dc817dfc52f7254abdce54abd4925aa834b4317d1de7335437fcd55b3cd6ce2324ee0d635d1fef936fa2da3dddc0efb52a5",
                    "0x99f9fc9c31719edafc796e58fac93e02f2284d4849f4d989f45e588dd488104194829cde75799d4278eba700b75a9527111f0c5d040553288ff99347fb0fa21a7e319085e04de7c7b539a9f0d243504131bdc1231558c7ac43daa13efe132192",
                    "0xac0e4aeadff832bdd9ef675581367c7aa40f7ce95fb065e055be071f73363e1e189eff57c897682bb00cd708360e8cf1091d8af594edf1f8901426cba1236e5ac30d3d0fe9b4196da68f1025d55fc8365060eead6735d5733a070eb634d2b895"
                  ],
                  "blsSignatures": [
                    "0x806d37897c9fdb0e5238385686e3d6c3cb5762e713310e149db606ee59db71148b075b89d78cfe3216b823eb4c21cdbf",
                    "",
                    "",
                    "0x970558ca105fc30f0af54ad811ea6d56d89713f39d27bc580eadad5bb5b454ac1cb57566bb5b697fe07f25c2c70be9b5",
                    "0x851ce7b4b49a2346c4957ea322724092e912852edb55a26c322a380f059a279b0f251820f808e283cc44ee5de57f006a",
                    "0x8ee7b15ab5a8eb63490969706701b9017c37f1aa75f511b239dbd657d2535807d5fc03e1110ddb56f6690f0b9039d76a"
                  ]
                }
              }
            ],
            "participantIds": [
              "eth|0xd9cbd194c9c90491b9e194d4b637f9cb80aa2259",
              "eth|0x447027e9ca54247f4972a18a87232b16b1a57598",
              "eth|0x00e7d0aacf98515e254fc9115667d026e57383f2",
              "eth|0xa3d64e14932754bc2385c01ffc683630bad1cebc",
              "eth|0x477b755dd510c3d10b78832bd58adf1325ccb783",
              "eth|0x60c05b5a734e8e44dfb4652c5d6b8226e79e1a92",
              "eth|0x25052aa09eee354df4f0d4d0b9c5401d771b7370"
            ],
            "participantEcdsaSignatures": [
              "0xdbf5b69e9a1f5c462e24ad400541eb43899b1b2ee5650256cce5d11291ae0f6459eeceddcf0907ab4620be6bfda390e47888f0fe7e0fb5c15f9bfeae7a8bcc9d1c",
              "0x1ba9cc2a6a16b43b14c853ce7afdeee3b2a82a0a3780ab550ed9edd9ff16c3611e0ef03fd13f35a924899b4eccde2c8ab9b3c90ab30b4fb8a756b3c25d447e091c",
              "",
              "",
              "0xc28b464ea0f5425483ff6e48b33044290065e93574451440fe41e5aaa0ef89bf6111030f3bd4596e1fc3dfeb9eaafd7f14975c1b6c10f8c923d5818b06d4d73c1b",
              "0x92a985831e3b701875c492ecc195051862cfcf37bdacf037b2ac4d4694d496c572d7ebff71dd61cd910d4ca6907dd22f439c0336a53ed6360226c35ed77a805a1b",
              "0x0e919ee070172fee2f1528ef25f554c80797f06cb6cc73f7c8a191a40cb6a84d150439d30fb21e7a7fbcef3e08a2cddd88e7a70447596e6468f3a7fd8776daa21b"
            ]
          });

        let u_id = Identity::eth_from_str("0x447027e9ca54247f4972a18a87232b16b1a57598").unwrap();
        let bt = serde_json::from_value::<BatchTranscript>(json).unwrap();

        // Verify pubkey sequence to end
        let result = verify_with_id::<DefaultEngine>(&bt, u_id);
        assert!(result.is_ok());

    }

}