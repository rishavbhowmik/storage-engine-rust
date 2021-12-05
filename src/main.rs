use std::fs::File;
use std::string::String;

struct Error {
    code: i32,
    message: String,
}
// add fmt::Debug trait to Error struct
use std::fmt;
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error {{ code: {}, message: {} }}",
            self.code, self.message
        )
    }
}

struct StorageHeader {
    block_len: u32,
}

const STORAGE_HEADER_SIZE: u32 = std::mem::size_of::<StorageHeader>() as u32;

struct BlockHeader {
    block_data_size: u32,
}

const BLOCK_HEADER_SIZE: u32 = std::mem::size_of::<BlockHeader>() as u32;

impl BlockHeader {
    fn new(block_data_size: u32) -> BlockHeader {
        BlockHeader {
            block_data_size: block_data_size,
        }
    }
}

use std::collections::BTreeSet;
struct Storage {
    header: StorageHeader,
    free_blocks: BTreeSet<u32>,
    file_writer: File,
    write_pointer: u64,
    file_reader: File,
    read_pointer: u64,
}

impl Storage {
    fn new(file_path: String, block_len: u32) -> Result<Storage, Error> {
        let const_file_path1 = file_path.clone();
        let const_file_path2 = file_path.clone();

        let file_create_result = File::create(const_file_path1);
        if file_create_result.is_err() {
            return Err(Error {
                code: 1,
                message: "Could not create file".to_string(),
            });
        }
        let file_open_result = File::open(const_file_path2);
        if file_open_result.is_err() {
            return Err(Error {
                code: 2,
                message: "Could not open file".to_string(),
            });
        }
        let mut storage = Storage {
            header: StorageHeader {
                block_len: block_len,
            },
            free_blocks: BTreeSet::new(),
            file_writer: file_create_result.unwrap(),
            write_pointer: 0,
            file_reader: file_open_result.unwrap(),
            read_pointer: 0,
        };
        let init_storage_result = storage.init_storage(storage.header.block_len);
        if init_storage_result.is_err() {
            return Err(Error {
                code: 2,
                message: "Could not init storage".to_string(),
            });
        }
        Ok(storage)
    }
    fn init_storage(&mut self, block_len: u32) -> Result<usize, Error> {
        use std::io::prelude::*;
        let file = &mut self.file_writer;
        let header_bytes = u32_to_bytes(block_len);
        let write_result = file.write(&header_bytes);
        if write_result.is_err() {
            return Err(Error {
                code: 2,
                message: "Could not write to file".to_string(),
            });
        }
        let file_size = write_result.unwrap();
        self.header.block_len = block_len;
        self.free_blocks.clear();
        self.write_pointer = file_size as u64;
        self.read_pointer = 0 as u64;
        Ok(file_size)
    }
    fn read_block(&mut self, block_index: usize) -> Result<(usize, Vec<u8>), Error> {
        use std::io::prelude::*;
        let block_size = self.header.block_len;
        let block_offset: usize = STORAGE_HEADER_SIZE as usize + block_index as usize * (BLOCK_HEADER_SIZE as usize + block_size as usize);
        // seek reader to block offset
        let seek_result = self
            .file_reader
            .seek(std::io::SeekFrom::Start(block_offset as u64));
        if seek_result.is_err() {
            return Err(Error {
                code: 3,
                message: "Could not seek to block offset".to_string(),
            });
        }
        // read block data length from inital 4 bytes
        let block_data_size_bytes = &mut [0u8; 4];
        let read_result = self.file_reader.read(block_data_size_bytes);
        if read_result.is_err() {
            return Err(Error {
                code: 3,
                message: "Could not read from file".to_string(),
            });
        }
        let _ = read_result.unwrap();
        let block_header = BlockHeader::new(bytes_to_u32(block_data_size_bytes));
        // read block data to vec
        let mut block_data = vec![0u8; block_header.block_data_size as usize];
        let read_result = self.file_reader.read(&mut block_data[..]);
        if read_result.is_err() {
            return Err(Error {
                code: 4,
                message: "Could not read from file".to_string(),
            });
        }
        let read_size = read_result.unwrap() as u32;
        let read_pointer: usize = block_offset as usize + BLOCK_HEADER_SIZE as usize + read_size as usize;
        // return read_pointer and block_data
        Ok((read_pointer, block_data))
    }
    fn write_block(&mut self, block_index: usize, data: Vec<u8>) -> Result<usize, Error> {
        use std::io::prelude::*;
        let block_size = self.header.block_len;
        let block_offset = STORAGE_HEADER_SIZE as usize + block_index as usize * ( BLOCK_HEADER_SIZE as usize + block_size as usize);
        // seek writer to block offset
        let seek_result = self
            .file_writer
            .seek(std::io::SeekFrom::Start(block_offset as u64));
        if seek_result.is_err() {
            return Err(Error {
                code: 5,
                message: "Could not seek to block offset".to_string(),
            });
        }
        // write block data length to inital 4 bytes
        let block_data_size_bytes = u32_to_bytes(data.len() as u32);
        let write_result = self.file_writer.write(&block_data_size_bytes);
        if write_result.is_err() {
            return Err(Error {
                code: 6,
                message: "Could not write to file".to_string(),
            });
        }
        let _ = write_result.unwrap();
        // write block data to file
        let write_result = self.file_writer.write(&data[..]);
        if write_result.is_err() {
            return Err(Error {
                code: 7,
                message: "Could not write to file".to_string(),
            });
        }
        let write_size = write_result.unwrap();
        let write_pointer: usize = block_offset as usize + BLOCK_HEADER_SIZE as usize + write_size as usize;
        // update block header
        let block_index = block_index as u32;
        self.free_blocks.remove(&block_index);
        // return write pointer
        Ok(write_pointer)
    }
}

// helper functions
fn u32_to_bytes(n: u32) -> ([u8; 4]) {
    // block_size is in bytes as little endian
    let mut bytes = [0u8; 4];
    bytes[3] = (n >> 24) as u8;
    bytes[2] = (n >> 16) as u8;
    bytes[1] = (n >> 8) as u8;
    bytes[0] = (n >> 0) as u8;
    bytes
}

fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut n: u32 = 0;
    n |= (bytes[0] as u32) << 0;
    n |= (bytes[1] as u32) << 8;
    n |= (bytes[2] as u32) << 16;
    n |= (bytes[3] as u32) << 24;
    n
}

fn main() {
    let mut storage = Storage::new("test.hex".to_string(), 8).unwrap();
    println!("{:?}", storage.free_blocks);
    println!("{:?}", storage.header.block_len);

    let data_sets = [u32_to_bytes(8), u32_to_bytes(16), u32_to_bytes(32)];

    let mut i = 0;
    for data in data_sets.iter() {
        let write_block_res = storage.write_block(i, data.to_vec());
        if write_block_res.is_err() {
            println!("{:?}", write_block_res.unwrap_err());
        } else {
            println!("{:?}", write_block_res.unwrap());
        }
        let read_block_res = storage.read_block(0);
        if read_block_res.is_err() {
            println!("{:?}", read_block_res.unwrap_err());
        } else {
            println!("{:?}", read_block_res.unwrap());
        }
        i+=1;
    }
}
