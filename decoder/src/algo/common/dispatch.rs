use bytes::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct DecoderParams {
    pub buffer: Bytes,
    pub extension: String,
}

#[derive(Clone)]
pub enum DecoderType {
    Raw,
    Ncm,
    Tm,
    Kgm,
    Kwm,
    Xm,
    Ximalaya,
    Qmc,
}

impl DecoderType {
    pub fn get_decoder(&self) -> Box<dyn super::DecoderBuilder> {
        match self {
            DecoderType::Raw => Box::new(super::raw::RawDecoderBuilder),
            DecoderType::Ncm => Box::new(super::super::ncm::NcmDecoderBuilder),
            DecoderType::Tm => Box::new(super::super::tm::TmDecoderBuilder),
            DecoderType::Kgm => Box::new(super::super::kgm::KgmDecoderBuilder),
            DecoderType::Kwm => Box::new(super::super::kwm::KwmDecoderBuilder),
            DecoderType::Xm => Box::new(super::super::xiami::XmDecoderBuilder),
            DecoderType::Ximalaya => Box::new(super::super::ximalaya::XimalayaDecoderBuilder),
            DecoderType::Qmc => Box::new(super::super::qmc::QmcDecoderBuilder),
        }
    }
}

pub struct DecoderMap(pub HashMap<String, Vec<(DecoderType, bool)>>);

impl DecoderMap {
    pub fn register(&mut self, ext: &str, noop: bool, decoder_type: DecoderType) {
        match self.0.get_mut(ext) {
            Some(v) => v.push((decoder_type, noop)),
            None => {
                self.0.insert(ext.to_string(), vec![(decoder_type, noop)]);
            }
        }
    }
    pub fn get(&self, ext: &str, skip_noop: bool) -> Vec<DecoderType> {
        if let Some(decoders) = self.0.get(ext) {
            if skip_noop {
                decoders
                    .iter()
                    .filter(|(_, noop)| !*noop)
                    .map(|(decoder_type, _)| decoder_type.clone())
                    .collect()
            } else {
                decoders
                    .iter()
                    .map(|(decoder_type, _)| decoder_type.clone())
                    .collect()
            }
        } else {
            Vec::new()
        }
    }
}

pub static DECODER_MAP: std::sync::OnceLock<DecoderMap> = std::sync::OnceLock::new();
pub fn get_static_decoder_map() -> &'static DecoderMap {
    DECODER_MAP.get_or_init(|| {
        use DecoderType::*;
        let mut map = DecoderMap(HashMap::new());
        map.register("mp3", true, Raw);
        map.register("flac", true, Raw);
        map.register("ogg", true, Raw);
        map.register("m4a", true, Raw);
        map.register("wav", true, Raw);
        map.register("wma", true, Raw);
        map.register("aac", true, Raw);
        // Kugou
        map.register("kgm", false, Kgm);
        map.register("kgma", false, Kgm);
        // Viper
        map.register("vpr", false, Kgm);
        // Kuwo Mp3/Flac
        map.register("kwm", false, Kwm);
        map.register("kwm", false, Raw);
        // Netease Mp3/Flac
        map.register("ncm", false, Ncm);
        // QQ Music IOS M4a (replace header)
        map.register("tm2", false, Tm);
        map.register("tm6", false, Tm);
        // QQ Music IOS Mp3 (not encrypted)
        map.register("tm0", false, Tm);
        map.register("tm3", false, Tm);
        // Xiami Wav/M4a/Mp3/Flac
        map.register("xm", false, Xm);
        // Xiami Typed Format
        map.register("wav", false, Xm);
        map.register("mp3", false, Xm);
        map.register("flac", false, Xm);
        map.register("m4a", false, Xm);
        // Ximalaya
        map.register("x2m", false, Ximalaya);
        map.register("x3m", false, Ximalaya);
        map.register("xm", false, Ximalaya);
        // QQ Music MP3
        map.register("qmc0", false, Qmc);
        map.register("qmc3", false, Qmc);
        // QQ Music M4A
        map.register("qmc2", false, Qmc);
        map.register("qmc4", false, Qmc);
        map.register("qmc6", false, Qmc);
        map.register("qmc8", false, Qmc);
        // QQ Music FLAC
        map.register("qmcflac", false, Qmc);
        // QQ Music OGG
        map.register("qmcogg", false, Qmc);
        // QQ Music Accompaniment M4A
        map.register("tkm", false, Qmc);
        // Moo Music
        map.register("bkcmp3", false, Qmc);
        map.register("bkcm4a", false, Qmc);
        map.register("bkcflac", false, Qmc);
        map.register("bkcwav", false, Qmc);
        map.register("bkcape", false, Qmc);
        map.register("bkcogg", false, Qmc);
        map.register("bkcwma", false, Qmc);
        // QQ Music Weiyun Flac
        map.register("666c6163", false, Qmc);
        // QQ Music Weiyun Mp3
        map.register("6d7033", false, Qmc);
        // QQ Music Weiyun Ogg
        map.register("6f6767", false, Qmc);
        // QQ Music Weiyun M4a
        map.register("6d3461", false, Qmc);
        // QQ Music Weiyun Wav
        map.register("776176", false, Qmc);
        // QQ Music New Ogg
        map.register("mgg", false, Qmc);
        map.register("mgg1", false, Qmc);
        map.register("mggl", false, Qmc);
        // QQ Music New Flac
        map.register("mflac", false, Qmc);
        map.register("mflac0", false, Qmc);
        map.register("mflach", false, Qmc);
        // QQ Music MP4 Container, tipically used for Dolby EAC3 stream
        map.register("mmp4", false, Qmc);
        map
    })
}
