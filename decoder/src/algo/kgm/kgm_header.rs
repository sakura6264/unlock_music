use crate::algo::DecoderResult;
const VPR_HEADER: [u8; 16] = [
    0x05, 0x28, 0xBC, 0x96, 0xE9, 0xE4, 0x5A, 0x43, 0x91, 0xAA, 0xBD, 0xD0, 0x7A, 0xF5, 0x36, 0x31,
];
const KGM_HEADER: [u8; 16] = [
    0x7C, 0xD5, 0x32, 0xEB, 0x86, 0x02, 0x7F, 0x4B, 0xA8, 0xAF, 0xA6, 0x8E, 0x0F, 0xFF, 0x99, 0x14,
];
#[derive(Clone, Default)]
pub struct Header {
    pub magic_header: [u8; 0x0f + 1], // 0x00-0x0f: magic header
    pub audio_offset: u32,            // 0x10-0x13: offset of audio data
    pub crypto_version: u32,          // 0x14-0x17: crypto version
    pub crypto_slot: u32,             // 0x18-0x1b: crypto key slot
    pub crypto_test_data: [u8; 0x2b - 0x1c + 1], // 0x1c-0x2b: crypto test data
    pub crypto_key: [u8; 0x3b - 0x2c + 1], // 0x2c-0x3b: crypto key
}

impl Header {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(buf: &[u8]) -> DecoderResult<Self> {
        if buf.len() < 0x3c {
            return Err("KgmHeader from_bytes error: Invalid Length".into());
        }
        let magic_header = buf[0x00..=0x0f].try_into().unwrap();
        if magic_header != KGM_HEADER && magic_header != VPR_HEADER {
            return Err("KgmHeader from_bytes error: Invalid Magic Header".into());
        }
        let audio_offset = u32::from_le_bytes(buf[0x10..=0x13].try_into().unwrap());
        let crypto_version = u32::from_le_bytes(buf[0x14..=0x17].try_into().unwrap());
        let crypto_slot = u32::from_le_bytes(buf[0x18..=0x1b].try_into().unwrap());
        let crypto_test_data = buf[0x1c..=0x2b].try_into().unwrap();
        let crypto_key = buf[0x2c..=0x3b].try_into().unwrap();
        Ok(Self {
            magic_header,
            audio_offset,
            crypto_version,
            crypto_slot,
            crypto_test_data,
            crypto_key,
        })
    }
}
