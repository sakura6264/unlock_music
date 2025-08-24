use super::super::super::internal::utils::bytes::*;

use super::super::DecoderResult;
use bytes::*;
use thiserror::Error;

pub struct Decoder {
    pub rd: EasyBytesWithCursor,
    pub cipher: Box<dyn super::super::Decrypter>,
    pub header: super::kgm_header::Header,
}

#[derive(Debug, Error)]
pub enum KgmDecoderError {
    #[error("KgmDecoder validate error: Unsupported crypto version")]
    UnsupportedCryptoVersion,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            rd: EasyBytesWithCursor::new(),
            cipher: Box::new(super::kgm_v3::KgmCryptoV3::default()),
            header: super::kgm_header::Header::default(),
        }
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
    // Validate checks if the file is a valid Kugou (.kgm, .vpr, .kgma) file.
    // rd will be seeked to the beginning of the encrypted audio.
    fn validate(&mut self) -> DecoderResult<()> {
        self.seek_start();
        let header_buf: [u8; 0x3c] = self.read_sized();
        let header = super::kgm_header::Header::from_bytes(&header_buf)?;
        // read start pos
        // prepare for read
        self.seek_start_next(header.audio_offset as usize);

        self.header = header.clone();
        match header.crypto_version {
            3 => {
                self.cipher = Box::new(super::kgm_v3::KgmCryptoV3::new(&header)?);
            }
            _ => {
                return Err(KgmDecoderError::UnsupportedCryptoVersion.into());
            }
        }

        Ok(())
    }
    fn decode_bytes(&mut self) -> DecoderResult<BytesMut> {
        let input_bytes = self.read_to_end();

        self.cipher.decrypt(input_bytes)
    }
}

#[derive(Clone)]
pub struct KgmDecoderBuilder;

impl super::super::DecoderBuilder for KgmDecoderBuilder {
    fn new_decoder(
        &self,
        p: &super::super::dispatch::DecoderParams,
    ) -> Box<dyn super::super::Decoder> {
        Box::new(Decoder {
            rd: EasyBytesWithCursor::create(p.buffer.clone()),
            cipher: Box::new(super::kgm_v3::KgmCryptoV3::default()),
            header: super::kgm_header::Header::default(),
        })
    }
}
