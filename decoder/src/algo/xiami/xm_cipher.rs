use crate::algo::DecoderResult;
use bytes::*;

#[derive(Default)]
pub struct XmCipher {
    mask: u8,
    encrypt_start_at: usize,
}

impl XmCipher {
    pub fn new(mask: u8, encrypt_start_at: usize) -> Self {
        Self {
            mask,
            encrypt_start_at,
        }
    }
}

impl super::super::Decrypter for XmCipher {
    fn check_uninit(&self) -> bool {
        false
    }

    fn decrypt(&mut self, data: Bytes) -> DecoderResult<BytesMut> {
        if self.check_uninit() {
            return Err("Cipher is not initialized".into());
        }
        let mut buf = BytesMut::from(data);
        for i in self.encrypt_start_at..buf.len() {
            buf[i] ^= self.mask;
        }
        Ok(buf)
    }
}
