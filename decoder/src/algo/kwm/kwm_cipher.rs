use crate::algo::DecoderResult;
use bytes::*;

#[derive(Clone, Default)]
pub struct KwmCipher {
    pub mask: [u8; 32],
}

impl KwmCipher {
    pub fn new(key: [u8; 8]) -> Self {
        let mask = Self::generate_mask(key);
        KwmCipher { mask }
    }

    pub fn generate_mask(key: [u8; 8]) -> [u8; 32] {
        let key_int = u64::from_le_bytes(key);
        let key_str = key_int.to_string();
        let key_bytes = Bytes::from(key_str.as_bytes().to_vec());
        // trim or pad into length 32
        let key_str_trim = super::pad_or_truncate(key_bytes, 32);
        let mut mask = [0u8; 32];
        for i in 0..32 {
            mask[i] = super::KEY_PREDEFINED[i] ^ key_str_trim[i]
        }
        mask
    }
}

impl crate::algo::Decrypter for KwmCipher {
    fn check_uninit(&self) -> bool {
        self.mask.iter().all(|&x| x == 0)
    }
    fn decrypt(&mut self, input: Bytes) -> DecoderResult<BytesMut> {
        if self.check_uninit() {
            return Err("Cipher is not initialized".into());
        }
        let mut buf = BytesMut::from(input);
        for i in 0..buf.len() {
            buf[i] ^= self.mask[i & 0x1F];
        }
        Ok(buf)
    }
}
