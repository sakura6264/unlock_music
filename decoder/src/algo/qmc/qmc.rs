use crate::algo::{DecoderParams, DecoderResult, Decrypter};
use crate::internal::utils::{BytesCursorHelper, EasyBytesWithCursor};
use bytes::*;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Clone)]
pub struct QmcDecoderBuilder;

impl super::super::DecoderBuilder for QmcDecoderBuilder {
    fn new_decoder(&self, p: &super::super::DecoderParams) -> Box<dyn super::super::Decoder> {
        Box::new(Decoder {
            raw: EasyBytesWithCursor::create(p.buffer.clone()),
            params: p.clone(),
            audio: EasyBytesWithCursor::new(),
            audio_len: 0,
            decode_key: Bytes::new(),
            cipher: Box::new(super::cipher_static::StaticCipher),

            song_id: 0,
            raw_mete_extract2: 0,

            album_id: 0,
            album_media_id: String::new(),
        })
    }
}
#[derive(Debug, Error)]
pub enum QmcDecoderError {
    #[error("QmcDecoder validate error: {0}")]
    Validate(String),
    #[error("QmcDecoder validate error: Invalid Audio Extension")]
    InvalidAudioExtension,
    #[error("QmcDecoder search_key error: {0}")]
    SearchKey(String),
    #[error("QmcDecoder search_key error: STag suffix doesn't contains media key")]
    InvalidSTag,
    #[error("QmcDecoder read_raw_key error: {0}")]
    ReadRawKey(String),
    #[error("QmcDecoder read_raw_meta_qtag invalid raw metadata: {0}")]
    InvalidRawMeta(String),
    #[error("QmcDecoder read_raw_meta_qtag invalid raw metadata")]
    InvalidRawMetaLen,
    #[error("QmcDecoder read_raw_meta_qtag invalid decode key: {0}")]
    InvalidDecodeKey(String),
    #[error("QmcDecoder read_raw_meta_qtag invalid song id: {0}")]
    InvalidSongId(String),
    #[error("QmcDecoder read_raw_meta_qtag invalid raw_mete_extract2: {0}")]
    InvalidRawMeteExtract2(String),
    #[error("QmcDecoder read error: Cipher Uninitialized")]
    CipherUninitialized,
}

pub struct Decoder {
    pub raw: EasyBytesWithCursor, // raw data
    pub params: DecoderParams,
    pub audio: EasyBytesWithCursor, // encrypted audio data
    pub audio_len: usize,
    pub decode_key: Bytes,
    pub cipher: Box<dyn Decrypter>,

    pub song_id: usize,
    pub raw_mete_extract2: usize,

    pub album_id: usize,
    pub album_media_id: String,
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

impl Decoder {
    pub fn validate_decode(&mut self) -> DecoderResult<()> {
        self.seek_start();
        let buf: [u8; 128] = self.read_sized();
        let buf = self
            .cipher
            .decrypt(Bytes::copy_from_slice(&buf))
            .map_err(|e| QmcDecoderError::Validate(e.to_string()))?;

        if crate::internal::sniff::audio_extension(&buf).is_none() {
            return Err(QmcDecoderError::InvalidAudioExtension.into());
        }
        Ok(())
    }

    pub fn search_key(&mut self) -> DecoderResult<()> {
        self.raw.seek_end_before(4);
        let file_size_m4 = self.raw.inner_cursor();
        let file_size = file_size_m4 + 4;

        let suffix_buf: [u8; 4] = self.read_sized();

        if suffix_buf.eq(b"QTag") {
            return self
                .read_raw_meta_qtag()
                .map_err(|e| QmcDecoderError::SearchKey(e.to_string()).into());
        } else if suffix_buf.eq(b"STag") {
            return Err(QmcDecoderError::InvalidSTag.into());
        }

        let size = u32::from_le_bytes(suffix_buf);
        if size <= 0xFFFF && size != 0 {
            return self.read_raw_key(size as usize);
        }
        self.audio_len = file_size;
        Ok(())
    }
    pub fn read_raw_key(&mut self, raw_key_len: usize) -> DecoderResult<()> {
        self.raw.seek_end_before(4 + raw_key_len);
        let audio_len = self.raw.inner_cursor();
        self.audio_len = audio_len;

        let mut raw_key_data = self.read(raw_key_len);
        if let Some(end) = raw_key_data.iter().rposition(|&x| x != b'\x00') {
            raw_key_data.truncate(end + 1);
        }
        let decoded_key = super::key_derive::derive_key(raw_key_data)
            .map_err(|e| QmcDecoderError::ReadRawKey(e.to_string()))?;
        self.decode_key = decoded_key.into();
        Ok(())
    }
    pub fn read_raw_meta_qtag(&mut self) -> DecoderResult<()> {
        self.raw.seek_end_before(8);
        let buf: [u8; 4] = self.read_sized();
        let raw_meta_len = u32::from_be_bytes(buf) as usize;
        self.raw.seek_end_before(8 + raw_meta_len);
        let audio_len = self.raw.inner_cursor();
        let raw_metadata = self.raw.read(raw_meta_len);
        let metadata = String::from_utf8(raw_metadata.to_vec())
            .map_err(|e| QmcDecoderError::InvalidRawMeta(e.to_string()))?;
        let items: Vec<String> = metadata.split(',').map(|s| s.to_string()).collect();
        if items.len() != 3 {
            return Err(QmcDecoderError::InvalidRawMetaLen.into());
        }
        self.decode_key =
            super::key_derive::derive_key(Bytes::copy_from_slice(items[0].as_bytes()))
                .map_err(|e| QmcDecoderError::InvalidDecodeKey(e.to_string()))?
                .into();
        self.song_id = items[1]
            .parse()
            .map_err(|e: ParseIntError| QmcDecoderError::InvalidSongId(e.to_string()))?;
        self.raw_mete_extract2 = items[2]
            .parse()
            .map_err(|e: ParseIntError| QmcDecoderError::InvalidRawMeteExtract2(e.to_string()))?;
        self.audio_len = audio_len;
        Ok(())
    }
}

impl super::super::Decoder for Decoder {
    fn validate(&mut self) -> DecoderResult<()> {
        self.search_key()
            .map_err(|e| QmcDecoderError::Validate(e.to_string()))?;
        if self.decode_key.len() > 300 {
            self.cipher = Box::new(super::cipher_rc4::Rc4Cipher::new(self.decode_key.clone()));
        } else if !self.decode_key.is_empty() {
            self.cipher = Box::new(
                super::cipher_map::MapCipher::new(self.decode_key.clone())
                    .map_err(|e| QmcDecoderError::Validate(e.to_string()))?,
            );
        } else {
            self.cipher = Box::new(super::cipher_static::StaticCipher);
        }

        self.validate_decode()
            .map_err(|e| QmcDecoderError::Validate(e.to_string()))?;
        self.raw.seek_start();
        self.audio = EasyBytesWithCursor::create(self.raw.read(self.audio_len));
        Ok(())
    }

    fn decode_bytes(&mut self) -> DecoderResult<BytesMut> {
        if self.cipher.check_uninit() {
            return Err(QmcDecoderError::CipherUninitialized.into());
        }
        let input = self.read_to_end();
        let output_buf = self
            .cipher
            .decrypt(input)
            .map_err(|e| QmcDecoderError::Validate(e.to_string()))?;
        Ok(output_buf)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::DecoderBuilder;

    use super::*;
    #[test]
    fn test_mflac0_decoder_read() {
        let mflac0_rc4_enc_body = include_bytes!("testdata/mflac0_rc4_raw.bin");
        let mflac0_rc4_enc_suffix = include_bytes!("testdata/mflac0_rc4_suffix.bin");
        let mflac0_rc4_source = [
            Bytes::from(mflac0_rc4_enc_body.as_slice()),
            Bytes::from(mflac0_rc4_enc_suffix.as_slice()),
        ]
        .concat();

        let mflac_rc4_enc_body = include_bytes!("testdata/mflac_rc4_raw.bin");
        let mflac_rc4_enc_suffix = include_bytes!("testdata/mflac_rc4_suffix.bin");
        let mflac_rc4_source = [
            Bytes::from(mflac_rc4_enc_body.as_slice()),
            Bytes::from(mflac_rc4_enc_suffix.as_slice()),
        ]
        .concat();

        let mflac_map_enc_body = include_bytes!("testdata/mflac_map_raw.bin");
        let mflac_map_enc_suffix = include_bytes!("testdata/mflac_map_suffix.bin");
        let mflac_map_source = [
            Bytes::from(mflac_map_enc_body.as_slice()),
            Bytes::from(mflac_map_enc_suffix.as_slice()),
        ]
        .concat();

        let mgg_map_enc_body = include_bytes!("testdata/mgg_map_raw.bin");
        let mgg_map_enc_suffix = include_bytes!("testdata/mgg_map_suffix.bin");
        let mgg_map_source = [
            Bytes::from(mgg_map_enc_body.as_slice()),
            Bytes::from(mgg_map_enc_suffix.as_slice()),
        ]
        .concat();

        let qmc0_static_enc_body = include_bytes!("testdata/qmc0_static_raw.bin");
        let qmc0_static_enc_suffix = include_bytes!("testdata/qmc0_static_suffix.bin");
        let qmc0_static_source = [
            Bytes::from(qmc0_static_enc_body.as_slice()),
            Bytes::from(qmc0_static_enc_suffix.as_slice()),
        ]
        .concat();

        let mut decoder_mflac0_rc4 =
            QmcDecoderBuilder.new_decoder(&super::super::super::DecoderParams {
                buffer: mflac0_rc4_source.into(),
                extension: ".flac".to_string(),
            });
        decoder_mflac0_rc4
            .validate()
            .map_err(|e| format!("QmcDecoder read error: {}", e))
            .unwrap();
        let mut decoder_mflac_rc4 =
            QmcDecoderBuilder.new_decoder(&super::super::super::DecoderParams {
                buffer: mflac_rc4_source.into(),
                extension: ".flac".to_string(),
            });
        decoder_mflac_rc4
            .validate()
            .map_err(|e| format!("QmcDecoder read error: {}", e))
            .unwrap();
        let mut decoder_mflac_map =
            QmcDecoderBuilder.new_decoder(&super::super::super::DecoderParams {
                buffer: mflac_map_source.into(),
                extension: ".flac".to_string(),
            });
        decoder_mflac_map
            .validate()
            .map_err(|e| format!("QmcDecoder read error: {}", e))
            .unwrap();
        let mut decoder_mgg_map =
            QmcDecoderBuilder.new_decoder(&super::super::super::DecoderParams {
                buffer: mgg_map_source.into(),
                extension: ".ogg".to_string(),
            });
        decoder_mgg_map
            .validate()
            .map_err(|e| format!("QmcDecoder read error: {}", e))
            .unwrap();
        let mut decoder_qmc0_static =
            QmcDecoderBuilder.new_decoder(&super::super::super::DecoderParams {
                buffer: qmc0_static_source.into(),
                extension: ".mp3".to_string(),
            });
        decoder_qmc0_static
            .validate()
            .map_err(|e| format!("QmcDecoder read error: {}", e))
            .unwrap();
    }
}
