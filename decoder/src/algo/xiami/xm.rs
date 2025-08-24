use super::super::super::internal::utils::bytes::*;
use crate::algo::DecoderResult;

use bytes::*;
use std::collections::HashMap;

const MAGIC_HEADER: [u8; 4] = [b'i', b'f', b'm', b't'];
const MAGIC_HEADER_2: [u8; 4] = [0xfe, 0xfe, 0xfe, 0xfe];
static TYPE_MAPPING: std::sync::OnceLock<HashMap<Bytes, String>> = std::sync::OnceLock::new();

pub fn get_type_mapping() -> &'static HashMap<Bytes, String> {
    TYPE_MAPPING.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert(b" WAV".to_vec().into(), "wav".to_string());
        map.insert(b"FLAC".to_vec().into(), "flac".to_string());
        map.insert(b" MP3".to_vec().into(), "mp3".to_string());
        map.insert(b" A4M".to_vec().into(), "m4a".to_string());
        map
    })
}

pub struct Decoder {
    pub rd: EasyBytesWithCursor,
    pub cipher: Box<dyn super::super::Decrypter>,
    pub output_ext: String,
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
        let header: [u8; 16] = self.read_sized();
        // 0x00 - 0x03 and 0x08 - 0x0B: magic header
        if !header[..4].eq(&MAGIC_HEADER) || !header[8..12].eq(&MAGIC_HEADER_2) {
            return Err("XmDecoder validate error: Invalid magic header".into());
        }
        if let Some(ext) = get_type_mapping().get(&header[4..8]) {
            self.output_ext = ext.clone();
        } else {
            return Err("XmDecoder validate error: Invalid audio extension".into());
        }
        // 0x0C - 0x0E, Encrypt Start At, LittleEndian Unit24
        let enc_start_at =
            (header[12] as u32) | (header[13] as u32) << 8 | (header[14] as u32) << 16;
        self.cipher = Box::new(super::xm_cipher::XmCipher::new(
            header[15],
            enc_start_at as usize,
        ));
        Ok(())
    }
    fn decode_bytes(&mut self) -> DecoderResult<BytesMut> {
        let input = self.read_to_end();
        self.cipher.decrypt(input)
    }
}

#[derive(Clone)]
pub struct XmDecoderBuilder;

impl super::super::DecoderBuilder for XmDecoderBuilder {
    fn new_decoder(&self, p: &super::super::DecoderParams) -> Box<dyn super::super::Decoder> {
        Box::new(Decoder {
            rd: EasyBytesWithCursor::create(p.buffer.clone()),
            cipher: Box::new(super::xm_cipher::XmCipher::default()),
            output_ext: String::new(),
        })
    }
}
