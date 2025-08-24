use crate::algo::DecoderResult;
use bytes::*;

#[derive(Clone, Default)]
pub struct KgmCryptoV3 {
    slot_box: [u8; 16],
    file_box: Vec<u8>,
}

const KGM_V3_SLOT2_KEY: [(u32, [u8; 4]); 1] = [(1, [0x6C, 0x2C, 0x2F, 0x27])];

impl KgmCryptoV3 {
    pub fn new(header: &super::kgm_header::Header) -> DecoderResult<Self> {
        let slot_key = if let Some((_, key)) = KGM_V3_SLOT2_KEY
            .iter()
            .find(|(id, _)| *id == header.crypto_slot)
        {
            key
        } else {
            return Err("KgmCryptoV3 new: Cannot find slot".into());
        };
        let slot_box = kugo_md5(slot_key);
        let mut file_box = kugo_md5(&header.crypto_key).to_vec();
        file_box.push(0x6b);
        Ok(Self { slot_box, file_box })
    }
}

impl crate::algo::Decrypter for KgmCryptoV3 {
    fn check_uninit(&self) -> bool {
        self.slot_box.iter().all(|&x| x == 0) || self.file_box.is_empty()
    }
    fn decrypt(&mut self, input: Bytes) -> DecoderResult<BytesMut> {
        if self.check_uninit() {
            return Err("Cipher is not initialized".into());
        }
        let mut buf = BytesMut::from(input);
        for i in 0..buf.len() {
            buf[i] ^= self.file_box[i % self.file_box.len()];
            buf[i] ^= buf[i] << 4;
            buf[i] ^= self.slot_box[i % self.slot_box.len()];
            buf[i] ^= xor_collapse_u32(i as u32);
        }
        Ok(buf)
    }
}

pub fn xor_collapse_u32(i: u32) -> u8 {
    let bytes = i.to_le_bytes();
    bytes[0] ^ bytes[1] ^ bytes[2] ^ bytes[3]
}

pub fn kugo_md5(b: &[u8]) -> [u8; 16] {
    use crypto::digest::Digest;
    let mut md5_instance = crypto::md5::Md5::new();
    let mut digest = vec![0u8; md5_instance.output_bytes()];
    md5_instance.input(b);
    md5_instance.result(&mut digest);
    let mut ret = [0u8; 16];
    let mut i = 0;
    while i < md5_instance.output_bytes() {
        ret[i] = digest[14 - i];
        ret[i + 1] = digest[14 - i + 1];
        i += 2;
    }
    ret
}
