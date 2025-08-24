use crate::algo::DecoderResult;
use bytes::*;

#[derive(Clone)]
pub struct Rc4Cipher {
    key: Bytes,
}

// the rust-crypto's implementation can't pass the test

impl Rc4Cipher {
    pub fn new(key: Bytes) -> Self {
        Self { key }
    }
}

impl super::super::Decrypter for Rc4Cipher {
    fn check_uninit(&self) -> bool {
        false
    }
    fn decrypt(&mut self, input: Bytes) -> DecoderResult<BytesMut> {
        if self.check_uninit() {
            return Err("Cipher is not initialized".into());
        }
        let mut rc4 = Rc4::new(&self.key, input);
        rc4.enc_first_seg();
        while rc4.enc_next_seg() {}
        let output = rc4.enc_out();
        Ok(output)
    }
}

// edit from here

#[derive(Clone)]
pub struct Rc4 {
    n: usize,
    state: Bytes,
    hash: u32,
    key: Bytes,
    segment_id: usize,
    data: BytesMut,
}

impl Rc4 {
    const RC4_FIRST_SEGMENT_SIZE: usize = 128;
    const RC4_SEGMENT_SIZE: usize = 5120;
    pub fn new(key: &[u8], input: Bytes) -> Self {
        // remove check size
        // assert!(key.len() >= 1 && key.len() <= 256);
        let mut rc4_state = BytesMut::zeroed(key.len());
        let n = key.len();
        for (i, x) in rc4_state.iter_mut().enumerate() {
            *x = (i & u8::MAX as usize) as u8;
        }
        let mut j = 0;
        for i in 0..n {
            j = (j + rc4_state[i] as usize + key[i % n] as usize) % n;
            rc4_state.swap(i, j);
        }
        let mut rc4 = Self {
            n,
            state: rc4_state.freeze(),
            hash: 0,
            key: Bytes::copy_from_slice(key),
            data: input.into(),
            segment_id: 0,
        };
        rc4.hash();
        rc4
    }
    fn enc_first_seg(&mut self) {
        for i in 0..Self::RC4_FIRST_SEGMENT_SIZE {
            let skip = self.get_segment_skip(i);
            self.data[i] ^= self.key[skip];
        }
        self.segment_id = 0;
    }
    fn enc_next_seg(&mut self) -> bool {
        // return false when end
        let not_end;
        let mut new_box = BytesMut::from(self.state.clone());
        let mut j = 0;
        let mut k = 0;
        let skiplen = self.get_segment_skip(self.segment_id);
        let seg_start = self.segment_id * Self::RC4_SEGMENT_SIZE;
        let seg_end_predict = (self.segment_id + 1) * Self::RC4_SEGMENT_SIZE;
        let seg_end = if seg_end_predict >= self.data.len() {
            not_end = false;
            self.data.len()
        } else {
            not_end = true;
            seg_end_predict
        };
        let seg_len = seg_end - seg_start;
        let mut i = -(skiplen as isize);
        while i < (seg_len as isize) {
            j = (j + 1) % self.n;
            k = (new_box[j] as usize + k) % self.n;
            new_box.swap(j, k);
            if i >= 0 && (seg_start + i as usize >= Self::RC4_FIRST_SEGMENT_SIZE) {
                self.data[seg_start + i as usize] ^=
                    new_box[(new_box[j] as usize + new_box[k] as usize) % self.n];
            }
            i += 1;
        }
        self.segment_id += 1;
        not_end
    }
    fn enc_out(&self) -> BytesMut {
        self.data.clone()
    }
    fn hash(&mut self) {
        self.hash = 1;
        for i in 0..self.n {
            let v = self.key[i] as u32;
            if v == 0 {
                continue;
            }
            let next_hash = self.hash.wrapping_mul(v);
            if next_hash == 0 || next_hash <= self.hash {
                break;
            }
            self.hash = next_hash;
        }
    }
    fn get_segment_skip(&self, id: usize) -> usize {
        let seed = self.key[id % self.n];
        let idx = (self.hash as f64) / ((id + 1) as f64 * seed as f64) * 100.0;
        (idx as usize) % self.n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc4() {
        use super::super::super::Decrypter;
        let mflac0_rc4_key = include_bytes!("testdata/mflac0_rc4_key.bin");
        let mflac0_rc4_raw = include_bytes!("testdata/mflac0_rc4_raw.bin");
        let mflac0_rc4_target = include_bytes!("testdata/mflac0_rc4_target.bin");

        let mut cipher = Rc4Cipher::new(Bytes::copy_from_slice(mflac0_rc4_key));
        let output = cipher
            .decrypt(Bytes::copy_from_slice(mflac0_rc4_raw))
            .unwrap();
        let target_bytes = Bytes::copy_from_slice(mflac0_rc4_target);
        assert_eq!(output, target_bytes);

        let mflac_rc4_key = include_bytes!("testdata/mflac_rc4_key.bin");
        let mflac_rc4_raw = include_bytes!("testdata/mflac_rc4_raw.bin");
        let mflac_rc4_target = include_bytes!("testdata/mflac_rc4_target.bin");
        let target_bytes = Bytes::copy_from_slice(mflac_rc4_target);
        let mut cipher = Rc4Cipher::new(Bytes::copy_from_slice(mflac_rc4_key));
        let output = cipher
            .decrypt(Bytes::copy_from_slice(mflac_rc4_raw))
            .unwrap();
        assert_eq!(output, target_bytes);
    }
}
