use crate::algo::DecoderResult;
use bytes::*;

pub struct MapCipher {
    pub key: Bytes,
    pub keybox: BytesMut,
    pub size: usize,
}

impl MapCipher {
    pub fn new(key: Bytes) -> DecoderResult<Self> {
        if key.is_empty() {
            return Err("MapCipher key is empty".into());
        }
        let c = Self {
            key: key.clone(),
            keybox: BytesMut::zeroed(key.len()),
            size: key.len(),
        };
        Ok(c)
    }
    pub fn rotate(value: u8, bits: u8) -> u8 {
        let rotate = bits.wrapping_add(4) % 8;
        let left = value << rotate;
        let right = value >> rotate;
        left | right
    }
    fn get_mask(&self, offset: usize) -> u8 {
        let mut tmp = offset;
        if tmp > 0x7fff {
            tmp %= 0x7fff;
        }
        let idx = (tmp * tmp + 71214) % self.size;
        Self::rotate(self.key[idx], idx as u8 & 0x7)
    }
}

impl super::super::Decrypter for MapCipher {
    fn check_uninit(&self) -> bool {
        self.keybox.is_empty() || self.key.is_empty()
    }
    fn decrypt(&mut self, input: Bytes) -> DecoderResult<BytesMut> {
        if self.check_uninit() {
            return Err("Cipher is not initialized".into());
        }
        let mut output = BytesMut::from(input);
        for i in 0..output.len() {
            output[i] ^= self.get_mask(i);
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_cipher() {
        use super::super::super::Decrypter;
        let mflac_map_key = include_bytes!("testdata/mflac_map_key.bin");
        let mflac_map_raw = include_bytes!("testdata/mflac_map_raw.bin");
        let mflac_map_target = include_bytes!("testdata/mflac_map_target.bin");
        let mut cipher = MapCipher::new(Bytes::copy_from_slice(mflac_map_key)).unwrap();
        let output = cipher
            .decrypt(Bytes::copy_from_slice(mflac_map_raw))
            .unwrap();
        let target_bytes = Bytes::copy_from_slice(mflac_map_target);
        assert_eq!(output, target_bytes);

        let mgg_map_key = include_bytes!("testdata/mgg_map_key.bin");
        let mgg_map_raw = include_bytes!("testdata/mgg_map_raw.bin");
        let mgg_map_target = include_bytes!("testdata/mgg_map_target.bin");
        let mut cipher = MapCipher::new(Bytes::copy_from_slice(mgg_map_key)).unwrap();
        let output = cipher.decrypt(Bytes::copy_from_slice(mgg_map_raw)).unwrap();
        let target_bytes = Bytes::copy_from_slice(mgg_map_target);
        assert_eq!(output, target_bytes);
    }
}
