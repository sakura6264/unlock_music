pub trait Sniffer {
    fn sniff(&self, header: &[u8]) -> bool;
}

// audio extension detection

pub fn audio_extension(header: &[u8]) -> Option<String> {
    if PrefixSniffer(b"ID3".to_vec()).sniff(header) {
        return Some(".mp3".to_string());
    }
    if PrefixSniffer(b"OggS".to_vec()).sniff(header) {
        return Some(".ogg".to_string());
    }
    if PrefixSniffer(b"RIFF".to_vec()).sniff(header) {
        return Some(".wav".to_string());
    }
    if PrefixSniffer(vec![
        0x30, 0x26, 0xb2, 0x75, 0x8e, 0x66, 0xcf, 0x11, 0xa6, 0xd9, 0x00, 0xaa, 0x00, 0x62, 0xce,
        0x6c,
    ])
    .sniff(header)
    {
        return Some(".wma".to_string()); // wma header
    }
    if M4aSniffer.sniff(header) {
        return Some(".m4a".to_string());
    }
    if Mpeg4Sniffer.sniff(header) {
        return Some(".mp4".to_string());
    }
    if PrefixSniffer(b"fLaC".to_vec()).sniff(header) {
        return Some(".flac".to_string());
    }
    if PrefixSniffer(b"FRM8".to_vec()).sniff(header) {
        return Some(".dff".to_string());
    }
    None
}

pub fn audio_extension_with_fallback(header: &[u8], fallback: String) -> String {
    let ext = audio_extension(header);
    ext.unwrap_or(fallback)
}

#[derive(Clone)]
pub struct PrefixSniffer(Vec<u8>);

impl Sniffer for PrefixSniffer {
    fn sniff(&self, header: &[u8]) -> bool {
        header.starts_with(&self.0)
    }
}

#[derive(Clone)]
pub struct M4aSniffer;

impl Sniffer for M4aSniffer {
    fn sniff(&self, header: &[u8]) -> bool {
        let mpeg4box = read_mpeg4_ftype_box(header);
        if let Some(mpeg4box) = mpeg4box {
            return mpeg4box.major_brand == "M4A "
                || mpeg4box.compatible_brands.contains(&"M4A ".to_string());
        }
        false
    }
}

#[derive(Clone)]
pub struct Mpeg4Sniffer;

impl Sniffer for Mpeg4Sniffer {
    fn sniff(&self, header: &[u8]) -> bool {
        read_mpeg4_ftype_box(header).is_some()
    }
}

#[derive(Clone)]
pub struct Mpeg4FtypeBox {
    pub major_brand: String,
    pub minor_version: u32,
    pub compatible_brands: Vec<String>,
}

pub fn read_mpeg4_ftype_box(header: &[u8]) -> Option<Mpeg4FtypeBox> {
    if (header.len() < 8) || !header[4..8].eq(b"ftyp") {
        return None;
    }

    let size = u32::from_be_bytes(header[0..4].try_into().unwrap());
    if size < 16 || size % 4 != 0 {
        return None;
    }

    let mpeg4box = Mpeg4FtypeBox {
        major_brand: String::from_utf8_lossy(&header[8..12]).to_string(),
        minor_version: u32::from_be_bytes(header[12..16].try_into().unwrap()),
        compatible_brands: header[16..(size as usize)]
            .chunks(4)
            .map(|c| String::from_utf8_lossy(c).to_string())
            .collect(),
    };
    Some(mpeg4box)
}

// image extension detection

pub fn image_mime(header: &[u8]) -> Option<String> {
    if PrefixSniffer(vec![0xFF, 0xD8, 0xFF]).sniff(header) {
        return Some("image/jpeg".to_string());
    }
    if PrefixSniffer(vec![b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n']).sniff(header) {
        return Some("image/png".to_string());
    }
    if PrefixSniffer(b"BM".to_vec()).sniff(header) {
        return Some("image/bmp".to_string());
    }
    if PrefixSniffer(b"RIFF".to_vec()).sniff(header) {
        return Some("image/webp".to_string());
    }
    if PrefixSniffer(b"GIF8".to_vec()).sniff(header) {
        return Some("image/gif".to_string());
    }
    None
}

pub fn image_extension(header: &[u8]) -> Option<String> {
    if PrefixSniffer(vec![0xFF, 0xD8, 0xFF]).sniff(header) {
        return Some(".jpg".to_string());
    }
    if PrefixSniffer(vec![b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n']).sniff(header) {
        return Some(".png".to_string());
    }
    if PrefixSniffer(b"BM".to_vec()).sniff(header) {
        return Some(".bmp".to_string());
    }
    if PrefixSniffer(b"RIFF".to_vec()).sniff(header) {
        return Some(".webp".to_string());
    }
    if PrefixSniffer(b"GIF8".to_vec()).sniff(header) {
        return Some(".gif".to_string());
    }
    None
}
