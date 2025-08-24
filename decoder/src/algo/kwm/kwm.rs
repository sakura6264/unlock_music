use super::super::super::internal::utils::bytes::*;

use super::super::DecoderResult;
use bytes::*;
use thiserror::Error;

pub const MAGIC_HEADER_1: &[u8; 16] = b"yeelion-kuwo-tme";
pub const MAGIC_HEADER_2: &[u8; 16] = b"yeelion-kuwo\x00\x00\x00\x00";
pub const KEY_PREDEFINED: &[u8; 32] = b"MoOtOiTvINGwd2E6n0E1i7L5t2IoOoNk";

pub struct Decoder {
    pub rd: EasyBytesWithCursor,
    pub cipher: Box<dyn super::super::Decrypter>,
    pub output_ext: String,
    pub bitrate: i32,
}

impl Decoder {
    pub fn get_audio_ext(&self) -> String {
        if self.output_ext.is_empty() {
            return String::new();
        }
        ".".to_string() + &self.output_ext
    }
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

impl super::super::Decoder for Decoder {
    fn validate(&mut self) -> DecoderResult<()> {
        // kwm header is fixed to 1024 bytes
        let header = self.read(0x400);
        // check magic header, 0x00 - 0x0F
        let magic_header = &header[0..0x10];
        if !magic_header.eq(MAGIC_HEADER_1) && !magic_header.eq(MAGIC_HEADER_2) {
            return Err(KwmDecoderError::InvalidMagicHeader.into());
        }
        self.cipher = Box::new(super::kwm_cipher::KwmCipher::new(
            header[0x18..0x20].try_into().unwrap(),
        ));

        (self.bitrate, self.output_ext) = parse_bitrate_and_type(header.slice(0x20..0x40));
        Ok(())
    }
    fn decode_bytes(&mut self) -> DecoderResult<BytesMut> {
        let input = self.read_to_end();
        self.cipher.decrypt(input)
    }
}

pub fn parse_bitrate_and_type(header: Bytes) -> (i32, String) {
    let mut index = header.len();
    while index != 0 && header[index - 1] == b'\x00' {
        index -= 1;
    }
    let tmp = header.slice(..index);
    let sep = tmp.iter().position(|&x| !x.is_ascii_digit()).unwrap();
    let bitrate = String::from_utf8_lossy(&tmp[..sep])
        .to_string()
        .parse()
        .unwrap();
    let output_ext = String::from_utf8_lossy(&tmp[sep..])
        .to_string()
        .to_lowercase();
    (bitrate, output_ext)
}

pub fn pad_or_truncate(raw: Bytes, length: usize) -> BytesMut {
    let len_raw = raw.len();
    let mut out = BytesMut::from(raw.clone());
    if len_raw == 0 {
        out = BytesMut::zeroed(len_raw);
    } else if len_raw > length {
        BytesMut::from(raw).truncate(length);
    } else if len_raw < length {
        let mut tmp = BytesMut::zeroed(length);

        for i in 0..tmp.len() {
            tmp[i] = raw[i % len_raw];
        }
        out = tmp;
    }
    out
}

#[derive(Debug, Error)]
pub enum KwmDecoderError {
    #[error("KwmDecoder validate: Invalid magic header")]
    InvalidMagicHeader,
}

#[derive(Clone)]
pub struct KwmDecoderBuilder;

impl super::super::DecoderBuilder for KwmDecoderBuilder {
    fn new_decoder(
        &self,
        p: &super::super::dispatch::DecoderParams,
    ) -> Box<dyn super::super::Decoder> {
        Box::new(Decoder {
            rd: EasyBytesWithCursor::create(p.buffer.clone()),
            cipher: Box::new(super::kwm_cipher::KwmCipher::default()),
            output_ext: String::new(),
            bitrate: 0,
        })
    }
}
