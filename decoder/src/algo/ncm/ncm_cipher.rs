use crate::algo::DecoderResult;
use bytes::*;

#[derive(Clone)]
pub struct NcmCipher {
    key: Vec<u8>,
    keybox: Vec<u8>,
}

impl NcmCipher {
    pub fn new(key: &[u8]) -> Self {
        let keybox = NcmCipher::build_keybox(key);
        NcmCipher {
            key: key.to_vec(),
            keybox,
        }
    }
    pub fn get_uninit() -> Self {
        // don't use it
        NcmCipher {
            key: Vec::new(),
            keybox: Vec::new(),
        }
    }

    pub fn build_keybox(key: &[u8]) -> Vec<u8> {
        let mut keybox = vec![0u8; 256];
        for i in 0u8..=255 {
            keybox[i as usize] = i;
        }
        let mut j = 0;
        for i in 0u8..=255 {
            j = keybox[i as usize]
                .wrapping_add(j)
                .wrapping_add(key[(i % key.len() as u8) as usize]);
            keybox.swap(i as usize, j as usize);
        }

        let mut ret = vec![0u8; 256];
        for i in 0u8..=255 {
            let i_ = i.wrapping_add(1);
            let si = keybox[i_ as usize];
            let sj = keybox[i_.wrapping_add(si) as usize];
            ret[i as usize] = keybox[si.wrapping_add(sj) as usize];
        }
        ret
    }
}

impl super::super::Decrypter for NcmCipher {
    fn decrypt(&mut self, input: Bytes) -> DecoderResult<BytesMut> {
        if self.check_uninit() {
            return Err("Cipher is not initialized".into());
        }
        let mut buf = BytesMut::from(input);
        for i in 0..buf.len() {
            buf[i] ^= self.keybox[i & 0xff];
        }
        Ok(buf)
    }
    fn check_uninit(&self) -> bool {
        self.key.is_empty() || self.keybox.is_empty()
    }
}
