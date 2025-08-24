use bytes::*;

pub const X2M_HEADER_SIZE: usize = 1024;
pub const X2M_KEY: [u8; 4] = [b'x', b'm', b'l', b'y'];
pub const X2M_SCRAMBLE_TABLE_BYTES: &[u8; 2048] = include_bytes!("x2m_scramble_table.bin");

static X2M_SCRAMBLE_TABLE: std::sync::OnceLock<[u16; X2M_HEADER_SIZE]> = std::sync::OnceLock::new();
pub fn get_x2m_scramble_table() -> &'static [u16; X2M_HEADER_SIZE] {
    X2M_SCRAMBLE_TABLE.get_or_init(|| {
        let mut table = [0u16; X2M_HEADER_SIZE];
        for i in 0..X2M_HEADER_SIZE {
            table[i] = u16::from_le_bytes(
                X2M_SCRAMBLE_TABLE_BYTES[i * 2..i * 2 + 2]
                    .try_into()
                    .unwrap(),
            );
        }
        table
    })
}

pub fn decrypt_x2m_header(src: Bytes) -> BytesMut {
    let mut dst = BytesMut::zeroed(src.len());
    for dst_idx in 0..src.len() {
        let src_idx = get_x2m_scramble_table()[dst_idx] as usize;
        dst[dst_idx] = src[src_idx] ^ X2M_KEY[dst_idx % X2M_KEY.len()];
    }
    dst
}
