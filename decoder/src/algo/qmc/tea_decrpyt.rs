// port from golang.org/x/crypto/tea package
use crate::algo::DecoderResult;
const BLOCK_SIZE: usize = 8;
const KEY_SIZE: usize = 16;
const DELTA: u32 = 0x9e3779b9;
const NUM_ROUNDS: usize = 64;

pub struct TeaCipher {
    key: [u8; KEY_SIZE],
    rounds: usize,
}

impl TeaCipher {
    pub fn new_with_rounds(key: &[u8; KEY_SIZE], rounds: usize) -> DecoderResult<TeaCipher> {
        if rounds & 1 != 0 {
            return Err("TeaCipher: rounds must be even".into());
        }
        Ok(TeaCipher { key: *key, rounds })
    }

    pub fn new(key: &[u8; KEY_SIZE]) -> DecoderResult<TeaCipher> {
        TeaCipher::new_with_rounds(key, NUM_ROUNDS)
    }

    pub fn block_size() -> usize {
        BLOCK_SIZE
    }

    pub fn decrypt_8_bytes(&self, input: &[u8; 8]) -> [u8; 8] {
        let mut v0 = u32::from_be_bytes(input[0..4].try_into().unwrap());
        let mut v1 = u32::from_be_bytes(input[4..8].try_into().unwrap());
        let k0 = u32::from_be_bytes(self.key[0..4].try_into().unwrap());
        let k1 = u32::from_be_bytes(self.key[4..8].try_into().unwrap());
        let k2 = u32::from_be_bytes(self.key[8..12].try_into().unwrap());
        let k3 = u32::from_be_bytes(self.key[12..16].try_into().unwrap());
        let mut sum = DELTA.wrapping_mul((self.rounds / 2) as u32);
        for _ in 0..self.rounds / 2 {
            v1 = v1.wrapping_sub(
                ((v0 << 4).wrapping_add(k2))
                    ^ (v0.wrapping_add(sum))
                    ^ ((v0 >> 5).wrapping_add(k3)),
            );
            v0 = v0.wrapping_sub(
                ((v1 << 4).wrapping_add(k0))
                    ^ (v1.wrapping_add(sum))
                    ^ ((v1 >> 5).wrapping_add(k1)),
            );
            sum = sum.wrapping_sub(DELTA);
        }
        let mut output = [0u8; 8];
        output[0..4].copy_from_slice(&v0.to_be_bytes());
        output[4..8].copy_from_slice(&v1.to_be_bytes());
        output
    }
}
