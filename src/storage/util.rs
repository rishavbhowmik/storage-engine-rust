/// convert 4 bytes unsinged integer little endian bytes array
pub fn u32_to_bytes(n: u32) -> ([u8; 4]) {
    // block_size is in bytes as little endian
    let mut bytes = [0u8; 4];
    bytes[3] = (n >> 24) as u8;
    bytes[2] = (n >> 16) as u8;
    bytes[1] = (n >> 8) as u8;
    bytes[0] = (n >> 0) as u8;
    bytes
}

/// convert little endian bytes array to 4 bytes unsinged integer
pub fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut n: u32 = 0;
    n |= (bytes[0] as u32) << 0;
    n |= (bytes[1] as u32) << 8;
    n |= (bytes[2] as u32) << 16;
    n |= (bytes[3] as u32) << 24;
    n
}
