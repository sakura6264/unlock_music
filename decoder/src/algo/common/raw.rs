use super::super::super::internal::utils::BytesCursorHelper;
use super::DecoderResult;
use bytes::*;

#[derive(Clone)]
pub struct RawDecoder {
    pub audio_ext: String,
    pub rd: Bytes,
    pub rd_cursor_offset: usize,
}

impl BytesCursorHelper for RawDecoder {
    fn inner_buffer(&self) -> Bytes {
        self.rd.clone()
    }
    fn inner_cursor(&self) -> usize {
        self.rd_cursor_offset
    }
    fn set_inner_cursor(&mut self, cursor: usize) {
        self.rd_cursor_offset = cursor;
    }
}

#[derive(Clone)]
pub struct RawDecoderBuilder;
impl super::DecoderBuilder for RawDecoderBuilder {
    fn new_decoder(&self, p: &super::dispatch::DecoderParams) -> Box<dyn super::Decoder> {
        Box::new(RawDecoder::new(p))
    }
}

impl RawDecoder {
    pub fn new(p: &super::dispatch::DecoderParams) -> RawDecoder {
        RawDecoder {
            audio_ext: p.extension.clone(),
            rd: p.buffer.clone(),
            rd_cursor_offset: 0,
        }
    }
}

impl std::io::Read for RawDecoder {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut locked_reader = std::io::Cursor::new(&self.rd);
        locked_reader.read(buf)
    }
}

impl super::Decoder for RawDecoder {
    fn validate(&mut self) -> DecoderResult<()> {
        use super::super::super::internal::sniff;
        self.seek_start();
        let header: [u8; 16] = self.read_sized();
        self.seek_start();
        let sniff_result = sniff::audio_extension(&header);
        if let Some(ext) = sniff_result {
            self.audio_ext = ext;
            Ok(())
        } else {
            Err("Audio extension not found".into())
        }
    }
    fn decode_bytes(&mut self) -> DecoderResult<BytesMut> {
        Ok(BytesMut::from(self.rd.clone()))
    }
}
