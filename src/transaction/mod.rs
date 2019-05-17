use crate::crypto::hash::{Hashable, H256};
use crate::crypto::sign::{KeyPair, PubKey, Signable, Signature};
use bincode::serialize;

/// A unique identifier of a transaction output, a.k.a. a coin.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CoinId {
    /// The hash of the transaction that produces this coin.
    pub hash: H256,
    /// The index of the coin in the output list of the transaction that produces this coin.
    pub index: u32,
}

impl CoinId {
    pub fn get_bytes(&self) -> u32 {
        return 36;
    }
}
/// An address of a user. It is the SHA256 hash of the user's public key.
pub type Address = H256;

/// An input of a transaction.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Input {
    /// The identifier of the input coin.
    pub coin: CoinId,
    /// The amount of this input.
    // TODO: this is redundant, since it is also stored in the transaction output. We need it to do
    // rollback.
    pub value: u64,
    /// The address of the owner of this input coin.
    // TODO: this is redundant, since it is also stored in the transaction output. We need it to do
    // rollback.
    pub owner: Address,
}


impl Input {
    pub fn get_bytes(&self) -> u32 {
        return self.coin.get_bytes()+8+32;
    }
}

/// An output of a transaction.
// TODO: coinbase output (transaction fee). Maybe we don't need that in this case.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Output {
    /// The amount of this output.
    pub value: u64,
    /// The address of the recipient of this output coin.
    pub recipient: Address,
}


impl Output {
    pub fn get_bytes(&self) -> u32 {
        return 8+32;
    }
}


/// A Prism transaction. It takes a set of existing coins (inputs) and transforms them into a set
/// of coins (outputs).
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// The list of inputs put into this transaction.
    pub input: Vec<Input>,
    /// The list of outputs generated by this transaction.
    pub output: Vec<Output>,
    /// Authorization of this transaction by the owners of the inputs.
    pub authorization: Vec<Authorization>,
}

impl Transaction {
    /// Return the size in bytes
    pub fn get_bytes(&self) -> u32 {
        let mut total_bytes = 0;
        for input in self.input.iter() {
            total_bytes += input.get_bytes();
        }
        for output in self.output.iter() {
            total_bytes += output.get_bytes();
        }
        for authorization in self.authorization.iter() {
            total_bytes += authorization.get_bytes();
        }
        return total_bytes;
    }
}


impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        return ring::digest::digest(&ring::digest::SHA256, &serialize(self).unwrap()).into();
    }
}

impl Signable for Transaction {
    fn sign(&self, keypair: &KeyPair) -> Signature {
        // note that we only want to sign the inputs and the outputs
        let raw_inputs = serialize(&self.input).unwrap();
        let raw_outputs = serialize(&self.output).unwrap();
        let raw = [&raw_inputs[..], &raw_outputs[..]].concat(); // we can also use Vec extend, don't know which is better
        keypair.sign(&raw)
    }

    fn verify(&self, public_key: &PubKey, signature: &Signature) -> bool {
        // note that we only sign the inputs and the outputs
        let raw_inputs = serialize(&self.input).unwrap();
        let raw_outputs = serialize(&self.output).unwrap();
        let raw = [&raw_inputs[..], &raw_outputs[..]].concat(); // we can also use Vec extend, don't know which is better
        public_key.verify(&raw, signature)
    }
}

/// Authorization of the transaction by the owner of an input coin.
#[derive(Serialize, Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub struct Authorization {
    /// The public key of the owner.
    pub pubkey: PubKey,
    /// The signature of the transaction input and output
    pub signature: Signature,
}

impl Authorization {
    /// Return the size in bytes
    pub fn get_bytes(&self) -> u32 {
        return 64;
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::crypto::hash::tests::generate_random_hash;
    use rand::{RngCore, Rng};

    pub fn generate_random_coinid() -> CoinId {
        let mut rng = rand::thread_rng();
        CoinId {
            hash: generate_random_hash(),
            index: rng.next_u32(),
        }
    }

    pub fn generate_random_input() -> Input {
        let mut rng = rand::thread_rng();
        Input {
            coin: generate_random_coinid(),
            value: rng.next_u64(),
            owner: generate_random_hash(),
        }
    }

    pub fn generate_random_output() -> Output {
        let mut rng = rand::thread_rng();
        Output {
            value: rng.next_u64(),
            recipient: generate_random_hash(),
        }
    }

    pub fn generate_random_transaction() -> Transaction {
        let mut rng = rand::thread_rng();
        Transaction {
            input: (0..rng.gen_range(1,5)).map(|_|generate_random_input()).collect(),
            output: (0..rng.gen_range(1,5)).map(|_|generate_random_output()).collect(),
            authorization: vec![],
        }
    }
}
