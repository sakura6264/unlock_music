use serde::Deserialize;
use serde_json::Value;

pub trait NcmMeta: super::super::AudioMeta {
    fn get_format(&self) -> String;
}

#[allow(dead_code)]
#[derive(Clone, Default, Deserialize)]
pub struct NcmMetaMusic {
    #[serde(rename = "format", default)]
    pub format: String,
    #[serde(rename = "musicName", default)]
    pub music_name: String,
    #[serde(rename = "artist", default)]
    pub artist: Vec<Vec<Value>>,
    #[serde(rename = "album", default)]
    pub album: String,
    #[serde(rename = "albumPicDocId", default)]
    pub album_pic_doc_id: Value,
    #[serde(rename = "albumPic", default)]
    pub album_pic: String,
    #[serde(rename = "flag", default)]
    pub flag: i32,
    #[serde(rename = "bitrate", default)]
    pub bitrate: i32,
    #[serde(rename = "duration", default)]
    pub duration: i32,
    #[serde(rename = "alias", default)]
    pub alias: Vec<Value>,
    #[serde(rename = "transNames", default)]
    pub trans_names: Vec<Value>,
}

impl NcmMeta for NcmMetaMusic {
    fn get_format(&self) -> String {
        self.format.clone()
    }
}

impl super::super::AudioMeta for NcmMetaMusic {
    fn get_title(&self) -> String {
        self.music_name.clone()
    }

    fn get_album(&self) -> String {
        self.album.clone()
    }
    fn get_artists(&self) -> Vec<String> {
        let mut output = Vec::new();
        for artist in &self.artist {
            for item in artist {
                if let Value::String(s) = item {
                    output.push(s.clone());
                }
            }
        }
        output
    }
    fn manual_clone(&self) -> Box<dyn super::super::AudioMeta> {
        Box::new(self.clone())
    }
}

#[allow(dead_code)]
#[derive(Clone, Default, Deserialize)]
pub struct NcmMetaDj {
    #[serde(rename = "programId", default)]
    pub program_id: i32,
    #[serde(rename = "programName", default)]
    pub program_name: String,
    #[serde(rename = "mainMusic", default)]
    pub main_music: NcmMetaMusic,
    #[serde(rename = "djId", default)]
    pub dj_id: i32,
    #[serde(rename = "djName", default)]
    pub dj_name: String,
    #[serde(rename = "djAvatarUrl", default)]
    pub dj_avatar_url: String,
    #[serde(rename = "createTime", default)]
    pub create_time: i64,
    #[serde(rename = "brand", default)]
    pub brand: String,
    #[serde(rename = "serial", default)]
    pub serial: String,
    #[serde(rename = "programDesc", default)]
    pub program_desc: String,
    #[serde(rename = "programFeeType", default)]
    pub program_fee_type: i32,
    #[serde(rename = "programBuyed", default)]
    pub program_buyed: bool,
    #[serde(rename = "radioId", default)]
    pub radio_id: i32,
    #[serde(rename = "radioName", default)]
    pub radio_name: String,
    #[serde(rename = "radioCategory", default)]
    pub radio_category: String,
    #[serde(rename = "radioCategoryId", default)]
    pub radio_category_id: i32,
    #[serde(rename = "radioDesc", default)]
    pub radio_desc: String,
    #[serde(rename = "radioFeeType", default)]
    pub radio_fee_type: i32,
    #[serde(rename = "radioFeeScope", default)]
    pub radio_fee_scope: i32,
    #[serde(rename = "radioBuyed", default)]
    pub radio_buyed: bool,
    #[serde(rename = "radioPrice", default)]
    pub radio_price: i32,
    #[serde(rename = "radioPurchaseCount", default)]
    pub radio_purchase_count: i32,
}

impl NcmMeta for NcmMetaDj {
    fn get_format(&self) -> String {
        self.main_music.get_format()
    }
}

impl super::super::AudioMeta for NcmMetaDj {
    fn get_title(&self) -> String {
        if self.program_name.is_empty() {
            self.radio_name.clone()
        } else {
            self.program_name.clone()
        }
    }

    fn get_album(&self) -> String {
        if self.brand.is_empty() {
            self.radio_category.clone()
        } else {
            self.brand.clone()
        }
    }
    fn get_artists(&self) -> Vec<String> {
        if self.dj_name.is_empty() {
            self.main_music.get_artists()
        } else {
            vec![self.dj_name.clone()]
        }
    }
    fn manual_clone(&self) -> Box<dyn super::super::AudioMeta> {
        Box::new(self.clone())
    }
}
