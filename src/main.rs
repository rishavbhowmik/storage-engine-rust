mod storage;
use storage::Storage;

fn main() {
    let mut storage = Storage::new("tmp/test.hex".to_string(), 8).unwrap();

    let data_sets = [
        u32_to_bytes(8),
        u32_to_bytes(16),
        u32_to_bytes(32),
        u32_to_bytes(11),
        u32_to_bytes(12),
        u32_to_bytes(13),
    ];

    let mut i = 0;
    for data in data_sets.iter() {
        let write_block_res = storage.write_block(i, data.to_vec());
        if write_block_res.is_err() {
            println!("{:?}", write_block_res.unwrap_err());
        } else {
            println!("{:?}", write_block_res.unwrap());
        }

        i += 1;
    }
    println!("Extra");
    let write_block_res = storage.write_block(i, [u32_to_bytes(14), u32_to_bytes(15)].concat());
    if write_block_res.is_err() {
        println!("{:?}", write_block_res.unwrap_err());
    } else {
        println!("{:?}", write_block_res.unwrap());
    }
    println!("delete till {}", storage.delete_block(2, false).unwrap());
    println!("delete till {}", storage.delete_block(3, false).unwrap());
    println!("delete till {}", storage.delete_block(2, true).unwrap());
    println!("delete till {}", storage.delete_block(3, true).unwrap());

    let mut i = 0; // skip first block
    for _ in data_sets.iter() {
        let read_block_res = storage.read_block(i);
        if read_block_res.is_err() {
            println!("{:?}", read_block_res.unwrap_err());
        } else {
            println!("{:?}", read_block_res.unwrap());
        }
        i += 1;
    }
    let read_block_res = storage.read_block(i);
    if read_block_res.is_err() {
        println!("{:?}", read_block_res.unwrap_err());
    } else {
        println!("{:?}", read_block_res.unwrap());
    }

    println!("Test open");
    let mut storage = Storage::open("tmp/test.hex".to_string()).unwrap();
    let mut i = 0; // skip first block
    for _ in data_sets.iter() {
        let read_block_res = storage.read_block(i);
        if read_block_res.is_err() {
            println!("{:?}", read_block_res.unwrap_err());
        } else {
            println!("{:?}", read_block_res.unwrap());
        }
        i += 1;
    }
    let read_block_res = storage.read_block(i);
    if read_block_res.is_err() {
        println!("{:?}", read_block_res.unwrap_err());
    } else {
        println!("{:?}", read_block_res.unwrap());
    }
    i += 1;
}

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
