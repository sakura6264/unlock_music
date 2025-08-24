use super::super::internal::utils::bytes::*;
use crate::algo::DecoderResult;

use bytes::*;

use bytes::Bytes;

const REPLACE_HEADER: [u8; 8] = [0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70];
const MAGIC_HEADER: [u8; 4] = [0x51, 0x51, 0x4D, 0x55];

pub struct Decoder {
    pub raw: EasyBytesWithCursor,
    pub result_buf: Bytes,
}

impl BytesCursorHelper for Decoder {
    fn inner_buffer(&self) -> Bytes {
        self.raw.inner_buffer()
    }
    fn inner_cursor(&self) -> usize {
        self.raw.inner_cursor()
    }
    fn set_inner_cursor(&mut self, cursor: usize) {
        self.raw.set_inner_cursor(cursor);
    }
}

impl super::Decoder for Decoder {
    fn validate(&mut self) -> DecoderResult<()> {
        let header: [u8; 8] = self.read_sized();
        let need_replace;
        if header[..MAGIC_HEADER.len()].eq(&MAGIC_HEADER) {
            need_replace = true;
        } else if super::super::internal::sniff::audio_extension(&header).is_some() {
            need_replace = false;
        } else {
            return Err("TmDecoder validate error: Invalid Header".into());
        }
        let m_buffer = self.read_to_end();
        // concat the header to buffer
        let mut header_buffer;
        if need_replace {
            header_buffer = REPLACE_HEADER.to_vec();
        } else {
            header_buffer = header.to_vec();
        }
        header_buffer.extend_from_slice(&m_buffer);
        // create bufreader
        self.result_buf = Bytes::from(header_buffer);
        Ok(())
    }
    fn decode_bytes(&mut self) -> DecoderResult<BytesMut> {
        if self.result_buf.is_empty() {
            return Err("TmDecoder read error: BufReader not initialized".into());
        }
        Ok(BytesMut::from(self.result_buf.clone()))
    }
}

#[derive(Clone)]
pub struct TmDecoderBuilder;

impl super::DecoderBuilder for TmDecoderBuilder {
    fn new_decoder(&self, p: &super::common::DecoderParams) -> Box<dyn super::Decoder> {
        Box::new(Decoder {
            raw: EasyBytesWithCursor::create(p.buffer.clone()),
            result_buf: Bytes::new(),
        })
    }
}
