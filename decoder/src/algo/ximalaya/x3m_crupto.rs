use bytes::*;
pub const X3M_KEY: [u8; 32] = [
    b'3', b'9', b'8', b'9', b'd', b'1', b'1', b'1', b'a', b'a', b'd', b'5', b'6', b'1', b'3', b'9',
    b'4', b'0', b'f', b'4', b'f', b'c', b'4', b'4', b'b', b'6', b'3', b'9', b'b', b'2', b'9', b'2',
];

pub const X3M_HEADER_SIZE: usize = 1024;
pub const X3M_SCRAMBLE_TABLE_BYTES: &[u8; 2048] = include_bytes!("x3m_scramble_table.bin");

static X3M_SCRAMBLE_TABLE: std::sync::OnceLock<[u16; X3M_HEADER_SIZE]> = std::sync::OnceLock::new();

fn get_x3m_scramble_table() -> &'static [u16; X3M_HEADER_SIZE] {
    X3M_SCRAMBLE_TABLE.get_or_init(|| {
        let mut table = [0u16; X3M_HEADER_SIZE];
        for i in 0..X3M_HEADER_SIZE {
            table[i] = u16::from_le_bytes(
                X3M_SCRAMBLE_TABLE_BYTES[i * 2..i * 2 + 2]
                    .try_into()
                    .unwrap(),
            );
        }
        table
    })
}

pub fn decrypt_x3m_header(src: Bytes) -> BytesMut {
    let mut dst = BytesMut::zeroed(src.len());
    for dst_idx in 0..src.len() {
        let src_idx = get_x3m_scramble_table()[dst_idx] as usize;
        dst[dst_idx] = src[src_idx] ^ X3M_KEY[dst_idx % X3M_KEY.len()];
    }
    dst
}
