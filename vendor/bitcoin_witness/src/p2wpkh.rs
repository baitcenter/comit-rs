use bitcoin_support::{Hash160, PubkeyHash, Script};
use secp256k1_support::KeyPair;
use witness::{UnlockParameters, Witness};

/// Utility function to generate the `prev_script` for a p2wpkh adddress.
/// A standard p2wpkh locking script of:
/// 00 14 <public_key_hash>
/// becomes
/// 19 76 a9 14 <public_key_hash> 88 ac
/// in the unlocking script. See BIP 143.
/// This function simply returns the latter as a Script.
fn generate_prev_script(public_key_hash: &PubkeyHash) -> Script {
    let public_key_hash: Hash160 = public_key_hash.clone().into();

    let mut prev_script = vec![0x76, 0xa9, 0x14];

    prev_script.append(&mut public_key_hash[..].to_vec());
    prev_script.push(0x88);
    prev_script.push(0xac);

    Script::from(prev_script)
}

pub trait UnlockP2wpkh {
    fn p2wpkh_unlock_parameters(&self) -> UnlockParameters;
}

impl UnlockP2wpkh for KeyPair {
    fn p2wpkh_unlock_parameters(&self) -> UnlockParameters {
        UnlockParameters {
            witness: vec![
                Witness::Signature(self.clone()),
                Witness::PublicKey(self.public_key().clone()),
            ],
            sequence: super::SEQUENCE_ALLOW_NTIMELOCK_NO_RBF,
            prev_script: generate_prev_script(&self.public_key().clone().into()),
        }
    }
}

#[cfg(test)]
mod test {
    use bitcoin_support::PrivateKey;
    use std::str::FromStr;
    extern crate hex;
    use super::*;
    #[test]
    fn correct_prev_script() {
        let private_key =
            PrivateKey::from_str("L4r4Zn5sy3o5mjiAdezhThkU37mcdN4eGp4aeVM4ZpotGTcnWc6k").unwrap();

        let keypair: KeyPair = private_key.secret_key().clone().into();
        let input_parameters = keypair.p2wpkh_unlock_parameters();
        // Note: You might expect it to be a is_p2wpkh() but it shouldn't be.
        assert!(
            input_parameters.prev_script.is_p2pkh(),
            "prev_script should be a p2pkh"
        );
    }

}
