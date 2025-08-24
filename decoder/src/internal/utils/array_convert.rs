use crate::algo::DecoderResult;
/// Safe array conversion utility to replace unwrap() calls
pub trait SafeArrayConvert<const N: usize> {
    fn try_into_array(self) -> DecoderResult<[u8; N]>;
}

impl<const N: usize> SafeArrayConvert<N> for &[u8] {
    fn try_into_array(self) -> DecoderResult<[u8; N]> {
        self.try_into()
            .map_err(|_| format!("Expected array of len {}, got {}", N, self.len()).into())
    }
}

impl<const N: usize> SafeArrayConvert<N> for Vec<u8> {
    fn try_into_array(self) -> DecoderResult<[u8; N]> {
        let len = self.len();
        self.try_into()
            .map_err(|_| format!("Expected array of len {}, got {}", N, len).into())
    }
}

/// Convert u32 from big-endian bytes safely
pub fn u32_from_be_bytes(bytes: &[u8]) -> DecoderResult<u32> {
    let array: [u8; 4] = bytes.try_into_array()?;
    Ok(u32::from_be_bytes(array))
}

/// Convert u16 from little-endian bytes safely  
pub fn u16_from_le_bytes(bytes: &[u8]) -> DecoderResult<u16> {
    let array: [u8; 2] = bytes.try_into_array()?;
    Ok(u16::from_le_bytes(array))
}
