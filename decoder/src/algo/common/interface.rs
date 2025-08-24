use bytes::*;

pub type DecoderResult<T> = Result<T, Box<dyn std::error::Error>>;

pub trait Decrypter {
    fn decrypt(&mut self, input: Bytes) -> DecoderResult<BytesMut>;
    fn check_uninit(&self) -> bool;
}

pub trait Decoder {
    fn validate(&mut self) -> DecoderResult<()>;
    fn decode_bytes(&mut self) -> DecoderResult<BytesMut>;
    fn get_cover_image(&mut self) -> Option<DecoderResult<Bytes>> {
        None
    }
    fn get_audio_meta(&self) -> Option<DecoderResult<Box<dyn AudioMeta>>> {
        None
    }
}

pub trait AudioMeta {
    fn get_artists(&self) -> Vec<String>;
    fn get_title(&self) -> String;
    fn get_album(&self) -> String;
    fn manual_clone(&self) -> Box<dyn AudioMeta>;
}

pub trait DecoderBuilder {
    fn new_decoder(&self, p: &super::dispatch::DecoderParams) -> Box<dyn Decoder>;
}
