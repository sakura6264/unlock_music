use super::super::super::internal::utils::bytes::*;
use super::super::DecoderResult;
use bytes::*;

pub struct Decoder {
    pub rd: EasyBytesWithCursor,
    pub audio: Bytes,
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
        use super::super::super::internal::sniff;
        let encrypted_header = self.read(super::x2m_crypto::X2M_HEADER_SIZE);
        {
            // try x2m
            let mut header = super::x2m_crypto::decrypt_x2m_header(encrypted_header.clone());
            if sniff::audio_extension(&header).is_some() {
                header.extend_from_slice(&self.read_to_end());
                self.audio = header.freeze();
                return Ok(());
            }
        }
        {
            // try x3m
            let mut header = super::x3m_crupto::decrypt_x3m_header(encrypted_header.clone());
            if sniff::audio_extension(&header).is_some() {
                header.extend_from_slice(&self.read_to_end());
                self.audio = header.freeze();
                return Ok(());
            }
        }

        Err("Ximalaya validate error: ximalaya: unknown format".into())
    }
    fn decode_bytes(&mut self) -> DecoderResult<BytesMut> {
        Ok(BytesMut::from(self.audio.clone()))
    }
}

#[derive(Clone)]
pub struct XimalayaDecoderBuilder;

impl super::super::DecoderBuilder for XimalayaDecoderBuilder {
    fn new_decoder(&self, p: &super::super::DecoderParams) -> Box<dyn super::super::Decoder> {
        Box::new(Decoder {
            rd: EasyBytesWithCursor::create(p.buffer.clone()),
            audio: Bytes::new(),
        })
    }
}
