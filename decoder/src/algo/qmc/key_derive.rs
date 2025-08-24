use crate::algo::DecoderResult;
use bytes::*;

pub fn simple_make_key(salt: u8, length: usize) -> BytesMut {
    let mut key_buf = BytesMut::zeroed(length);
    for i in 0..length {
        let tmp = f64::tan((salt as f64) + (i as f64) * 0.1);
        key_buf[i] = (tmp.abs() * 100.0) as u8;
    }
    key_buf
}

pub const RAW_KEY_PREFIX_V2: &[u8; 18] = b"QQMusic EncV2,Key:";

pub fn derive_key(raw_key: Bytes) -> DecoderResult<BytesMut> {
    use base64::prelude::*;
    let raw_key_dec = BASE64_STANDARD
        .decode(raw_key)
        .map_err(|e| super::QmcDecoderError::ReadRawKey(format!("Base64 decode failed: {}", e)))?;
    let raw_key_dec = Bytes::from(raw_key_dec);
    let mut output_key = BytesMut::from(raw_key_dec.clone());
    if check_prefix(&raw_key_dec, RAW_KEY_PREFIX_V2) {
        output_key = derive_key_v2(raw_key_dec.slice(RAW_KEY_PREFIX_V2.len()..))?;
    }
    derive_key_v1(output_key.freeze())
}

pub fn derive_key_v1(raw_key_dec: Bytes) -> DecoderResult<BytesMut> {
    if raw_key_dec.len() < 16 {
        return Err("Derive key v1: raw key too short".into());
    }

    let simple_key = simple_make_key(106, 8);
    let mut tea_key = [0u8; 16];
    for i in 0..8 {
        tea_key[i << 1] = simple_key[i];
        tea_key[(i << 1) + 1] = raw_key_dec[i];
    }
    let rs = decrypt_tencent_tea(raw_key_dec.slice(8..), &tea_key)?;
    let mut ret = BytesMut::with_capacity(8 + rs.len());
    ret.extend_from_slice(&raw_key_dec[..8]);
    ret.extend_from_slice(&rs);
    Ok(ret)
}

const DERIVE_V2_KEY_1: [u8; 16] = [
    0x33, 0x38, 0x36, 0x5A, 0x4A, 0x59, 0x21, 0x40, 0x23, 0x2A, 0x24, 0x25, 0x5E, 0x26, 0x29, 0x28,
];
const DERIVE_V2_KEY_2: [u8; 16] = [
    0x2A, 0x2A, 0x23, 0x21, 0x28, 0x23, 0x24, 0x25, 0x26, 0x5E, 0x61, 0x31, 0x63, 0x5A, 0x2C, 0x54,
];

pub fn derive_key_v2(raw_key_dec: Bytes) -> DecoderResult<BytesMut> {
    use base64::prelude::*;
    let buf = decrypt_tencent_tea(raw_key_dec, &DERIVE_V2_KEY_1)?;
    let buf = decrypt_tencent_tea(buf.freeze(), &DERIVE_V2_KEY_2)?;
    let buf = BASE64_STANDARD
        .decode(buf)
        .map_err(|e| super::QmcDecoderError::ReadRawKey(format!("Base64 decode failed: {}", e)))?;
    let buf_bytes = Bytes::from(buf);
    Ok(BytesMut::from(buf_bytes))
}

pub fn check_prefix(input: &[u8], prefix: &[u8]) -> bool {
    prefix.eq(&input[0..prefix.len()])
}

pub fn xor_8_bytes(a: &[u8; 8], b: &[u8; 8]) -> [u8; 8] {
    let mut dst = [0u8; 8];
    for i in 0..8 {
        dst[i] = a[i] ^ b[i];
    }
    dst
}

pub fn decrypt_tencent_tea(inbuf: Bytes, key: &[u8; 16]) -> DecoderResult<BytesMut> {
    const SALT_LEN: usize = 2;
    const ZERO_LEN: usize = 7;
    if inbuf.len() % 8 != 0 {
        return Err("inbuf size not a multiple of the block size".into());
    }
    if inbuf.len() < 16 {
        return Err("inbuf size too small".into());
    }

    let blk = super::tea_decrpyt::TeaCipher::new_with_rounds(key, 32)?;
    let mut destbuf = blk.decrypt_8_bytes(&inbuf[0..8].try_into().unwrap());
    let padlen = (destbuf[0] & 0x7) as usize;
    let outlen = inbuf.len() - 1 - padlen - SALT_LEN - ZERO_LEN;
    let mut out = BytesMut::zeroed(outlen);
    let mut iv_prev = [0u8; 8];
    let mut iv_cur = inbuf[0..8].try_into().unwrap();

    let mut in_buf_pos = 8;
    let mut dest_idx = 1 + padlen;
    macro_rules! crypt_block {
        () => {
            iv_prev = iv_cur;
            iv_cur = inbuf[in_buf_pos..in_buf_pos + 8].try_into().unwrap();
            destbuf = xor_8_bytes(
                &destbuf,
                inbuf[in_buf_pos..in_buf_pos + 8].try_into().unwrap(),
            );
            destbuf = blk.decrypt_8_bytes(&destbuf);
            in_buf_pos += 8;
            dest_idx = 0;
        };
    }
    let mut i = 1;
    while i <= SALT_LEN {
        if dest_idx < 8 {
            dest_idx += 1;
            i += 1;
        } else if dest_idx == 8 {
            crypt_block!();
        }
    }
    let mut outpos = 0;
    while outpos < outlen {
        if dest_idx < 8 {
            out[outpos] = destbuf[dest_idx] ^ iv_prev[dest_idx];
            dest_idx += 1;
            outpos += 1;
        } else if dest_idx == 8 {
            crypt_block!();
        }
    }

    for i in 1..=ZERO_LEN {
        if destbuf[i] != iv_prev[i] {
            return Err("Key Derive: zero check fail".into());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_make_key() {
        let expect = [0x69u8, 0x56, 0x46, 0x38, 0x2b, 0x20, 0x15, 0x0b];
        let got = simple_make_key(106, 8);
        assert_eq!(expect.to_vec(), got.to_vec());
    }

    #[test]
    fn test_decrypt_key() {
        let mflac0_rc4_key_raw = include_bytes!("testdata/mflac0_rc4_key_raw.bin");
        let mflac0_rc4_key = include_bytes!("testdata/mflac0_rc4_key.bin");
        let output = derive_key(Bytes::copy_from_slice(mflac0_rc4_key_raw)).unwrap();
        let target_bytes = Bytes::copy_from_slice(mflac0_rc4_key);
        assert_eq!(output, target_bytes);
        let mflac_map_key_raw = include_bytes!("testdata/mflac_map_key_raw.bin");
        let mflac_map_key = include_bytes!("testdata/mflac_map_key.bin");
        let output = derive_key(Bytes::copy_from_slice(mflac_map_key_raw)).unwrap();
        let target_bytes = Bytes::copy_from_slice(mflac_map_key);
        assert_eq!(output, target_bytes);
        let mflac_rc4_key_raw = include_bytes!("testdata/mflac_rc4_key_raw.bin");
        let mflac_rc4_key = include_bytes!("testdata/mflac_rc4_key.bin");
        let output = derive_key(Bytes::copy_from_slice(mflac_rc4_key_raw)).unwrap();
        let target_bytes = Bytes::copy_from_slice(mflac_rc4_key);
        assert_eq!(output, target_bytes);
        let mgg_map_key_raw = include_bytes!("testdata/mgg_map_key_raw.bin");
        let mgg_map_key = include_bytes!("testdata/mgg_map_key.bin");
        let output = derive_key(Bytes::copy_from_slice(mgg_map_key_raw)).unwrap();
        let target_bytes = Bytes::copy_from_slice(mgg_map_key);
        assert_eq!(output, target_bytes);
    }
}
