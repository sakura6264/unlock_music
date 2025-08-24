use super::super::super::internal::utils::bytes::*;

use super::super::DecoderResult;
use bytes::*;
use thiserror::Error;

use super::meta;

const MAGISK_HEADER: &[u8; 8] = b"CTENFDAM";
const KEY_CORE: [u8; 16] = [
    0x68, 0x7a, 0x48, 0x52, 0x41, 0x6d, 0x73, 0x6f, 0x35, 0x6b, 0x49, 0x6e, 0x62, 0x61, 0x78, 0x57,
];
const KEY_META: [u8; 16] = [
    0x23, 0x31, 0x34, 0x6C, 0x6A, 0x6B, 0x5F, 0x21, 0x5C, 0x5D, 0x26, 0x30, 0x55, 0x3C, 0x27, 0x28,
];

#[derive(Clone)]
pub struct NcmDecoderBuilder;

impl super::super::DecoderBuilder for NcmDecoderBuilder {
    fn new_decoder(&self, p: &super::super::DecoderParams) -> Box<dyn super::super::Decoder> {
        Box::new(Decoder {
            rd: EasyBytesWithCursor::create(p.buffer.clone()),
            cipher: Box::new(super::ncm_cipher::NcmCipher::get_uninit()),
            meta_raw: Vec::new(),
            meta_type: String::new(),
            meta: Box::new(meta::NcmMetaMusic::default()),
            cover: Bytes::new(),
        })
    }
}

#[derive(Debug, Error)]
pub enum NcmDecoderError {
    #[error("NcmDecoder validate error: Magic header mismatch")]
    MagicHeaderMismatch,
    #[error("NcmDecoder read_meta_data error: Base64 Decode: {0}")]
    Base64Decode(String),
    #[error("NcmDecoder read_meta_data error: Meta Type not found")]
    MetaTypeNotFound,
    #[error("NcmDecoder parse_meta error: Parse Meta: {0}")]
    ParseMeta(String),
    #[error("NcmDecoder parse_meta error: Unknown Meta Type")]
    UnknownMetaType,
    #[error("NcmDecoder read error: Cipher Uninitialized")]
    CipherUninitialized,
    #[error("NcmDecoder crypto error: {0}")]
    Crypto(String),
}

pub struct Decoder {
    pub rd: EasyBytesWithCursor,
    pub cipher: Box<dyn super::super::Decrypter>,
    pub meta_raw: Vec<u8>,
    pub meta_type: String,
    pub meta: Box<dyn super::meta::NcmMeta>,
    pub cover: Bytes,
}

impl BytesCursorHelper for Decoder {
    fn inner_buffer(&self) -> Bytes {
        self.rd.inner_buffer()
    }
    fn inner_cursor(&self) -> usize {
        self.rd.inner_cursor()
    }
    fn set_inner_cursor(&mut self, cursor: usize) {
        self.rd.set_inner_cursor(cursor);
    }
}

impl Decoder {
    pub fn validate_magic_header(&mut self) -> DecoderResult<()> {
        let header: [u8; MAGISK_HEADER.len()] = self.read_sized();
        if !header.eq(MAGISK_HEADER) {
            return Err(NcmDecoderError::MagicHeaderMismatch.into());
        }
        Ok(())
    }

    pub fn read_key_data(&mut self) -> DecoderResult<Vec<u8>> {
        let b_key_len: [u8; 4] = self.read_sized();
        let i_key_len = u32::from_le_bytes(b_key_len);
        let mut b_key_raw = BytesMut::from(self.read(i_key_len as usize));
        for i in 0..i_key_len as usize {
            b_key_raw[i] ^= 0x64;
        }
        use super::super::super::internal::utils::*;
        let aes128ecb_result = decrypt_aes128ecb(&b_key_raw, &KEY_CORE)?;
        let pkcs7_result = pkcs7_unpadding(&aes128ecb_result);
        let output_result = pkcs7_result[17..].to_vec();
        Ok(output_result)
    }
    pub fn read_meta_data(&mut self) -> DecoderResult<()> {
        let b_meta_len: [u8; 4] = self.read_sized();
        let i_meta_len = u32::from_le_bytes(b_meta_len);
        if i_meta_len == 0 {
            // no meta data
            return Ok(());
        }
        let b_meta_raw = self.read(i_meta_len as usize);
        // remove first 22 bytes "163 key(Don't modify):"
        let mut b_meta_raw = b_meta_raw[22..].to_vec();
        for i in 0..b_meta_raw.len() {
            b_meta_raw[i] ^= 0x63;
        }
        use super::super::super::internal::utils::*;
        use base64::prelude::*;
        let cipher_text = BASE64_STANDARD
            .decode(b_meta_raw)
            .map_err(|e| NcmDecoderError::Base64Decode(e.to_string()))?;
        let cipher_text_aes128ecb =
            decrypt_aes128ecb(&cipher_text, &KEY_META).map_err(NcmDecoderError::Crypto)?;
        let meta_raw = pkcs7_unpadding(&cipher_text_aes128ecb);
        let sep = meta_raw.iter().position(|&x| x == b':');
        if let Some(sep) = sep {
            self.meta_type = String::from_utf8_lossy(&meta_raw[..sep]).to_string();
            self.meta_raw = meta_raw[sep + 1..].to_vec();
        } else {
            return Err(NcmDecoderError::MetaTypeNotFound.into());
        }
        Ok(())
    }
    pub fn read_cover_data(&mut self) -> DecoderResult<()> {
        let _b_cover_crc: [u8; 4] = self.read_sized();
        let b_cover_len: [u8; 4] = self.read_sized();
        let i_cover_len = u32::from_le_bytes(b_cover_len);
        let cover_buf = self.read(i_cover_len as usize);
        self.cover = cover_buf;
        Ok(())
    }
    pub fn parse_meta(&mut self) -> DecoderResult<()> {
        match self.meta_type.as_str() {
            "music" => {
                let meta: meta::NcmMetaMusic = serde_json::from_slice(&self.meta_raw)
                    .map_err(|e| NcmDecoderError::ParseMeta(e.to_string()))?;
                self.meta = Box::new(meta);
            }
            "dj" => {
                let meta: meta::NcmMetaDj = serde_json::from_slice(&self.meta_raw)
                    .map_err(|e| NcmDecoderError::ParseMeta(e.to_string()))?;
                self.meta = Box::new(meta);
            }
            _ => {
                return Err(NcmDecoderError::UnknownMetaType.into());
            }
        }
        Ok(())
    }
    pub fn get_audio_ext(&self) -> String {
        let format = self.meta.get_format();
        if !format.is_empty() {
            return ".".to_string() + &format;
        }
        String::new()
    }
}

impl super::super::Decoder for Decoder {
    fn validate(&mut self) -> DecoderResult<()> {
        self.validate_magic_header()?;
        // 2 bytes gap
        self.seek_next(2);
        let key_data = self.read_key_data()?;
        self.read_meta_data()?;
        // 5 bytes gap
        self.seek_next(5);
        self.read_cover_data()?;
        self.parse_meta()?;
        self.cipher = Box::new(super::ncm_cipher::NcmCipher::new(&key_data));
        Ok(())
    }
    fn decode_bytes(&mut self) -> DecoderResult<BytesMut> {
        if self.cipher.check_uninit() {
            return Err(NcmDecoderError::CipherUninitialized.into());
        }
        let input = self.read_to_end();
        self.cipher.decrypt(input)
    }

    fn get_cover_image(&mut self) -> Option<DecoderResult<Bytes>> {
        Some(Ok(self.cover.clone()))
    }

    fn get_audio_meta(&self) -> Option<DecoderResult<Box<dyn super::super::AudioMeta>>> {
        Some(Ok(self.meta.manual_clone()))
    }
}
