use {
    sha2::{Digest, Sha256}
};

#[derive(Clone, Default)]
struct Hasher {
    hasher: Sha256,
}

impl Hasher {
    pub fn hash(&mut self, val: &[u8]) {
        self.hasher.update(val);
    }
    pub fn hashv(&mut self, vals: &[&[u8]]) {
        for val in vals {
            self.hash(val);
        }
    }
    pub fn result(self) -> [u8; 32] {
        <[u8; 32]>::try_from(self.hasher.finalize().as_slice()).unwrap()
    }
}

fn hashv(vals: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Hasher::default();
    hasher.hashv(vals);
    hasher.result()
}

fn hash(val: &[u8]) -> [u8; 32] {
    hashv(&[val])
}

fn sighash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);

    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(&hash(preimage.as_bytes())[..8]);
    sighash
}

pub fn get_ix_data(name: &str, mut ix_data: Vec<u8>) -> Vec<u8> {
    let mut data = sighash("global", name).to_vec();
    data.append(&mut ix_data);
    data
}