mod error;
use error::Error;
mod util;
use util::*;

/// Main Header for storage file
/// - Stores constant capacity of each block as 4 bytes unsied integer as little endian
struct StorageHeader {
    block_len: u32,
}

const STORAGE_HEADER_SIZE: u32 = std::mem::size_of::<StorageHeader>() as u32;

/// Header of each block
/// - Stores size of data stored in the block as 4 bytes unsied integer as little endian
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
use std::fs::{File, OpenOptions};

pub struct Storage {
    header: StorageHeader,
    free_blocks: BTreeSet<u32>,
    file_writer: File,
    write_pointer: u64,
    file_reader: File,
    read_pointer: u64,
}

impl Storage {
    fn open_new_file_writer(file_path: &String) -> Result<(File, u64), Error> {
        let file_path_clone = file_path.clone();
        let file_writer_result = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path_clone);
        if file_writer_result.is_err() {
            return Err(Error {
                code: 1,
                message: "Could not create file".to_string(),
            });
        }
        let file_writer = file_writer_result.unwrap();
        let write_pointer = 0 as u64;
        Ok((file_writer, write_pointer))
    }

    fn open_file_reader(file_path: &String) -> Result<(File, u64), Error> {
        let file_path_clone = file_path.clone();
        let file_reader_result = OpenOptions::new().read(true).open(file_path_clone);
        if file_reader_result.is_err() {
            return Err(Error {
                code: 1,
                message: "Could not open file".to_string(),
            });
        }
        let file_reader = file_reader_result.unwrap();
        let read_pointer = 0 as u64;
        Ok((file_reader, read_pointer))
    }

    /// Create new storage file
    /// - Create/Overwrite new storage file in given path
    /// - Initializes storage header
    pub fn new(file_path: String, block_len: usize) -> Result<Storage, Error> {
        let file_writer = Storage::open_new_file_writer(&file_path);
        if file_writer.is_err() {
            return Err(file_writer.unwrap_err());
        }
        let (file_writer, write_pointer) = file_writer.unwrap();

        let file_reader = Storage::open_file_reader(&file_path);
        if file_reader.is_err() {
            return Err(file_reader.unwrap_err());
        }
        let (file_reader, read_pointer) = file_reader.unwrap();

        let mut storage = Storage {
            header: StorageHeader {
                block_len: block_len as u32,
            },
            free_blocks: BTreeSet::new(),
            file_writer,
            write_pointer,
            file_reader,
            read_pointer,
        };
        let init_storage_result = storage.set_storage_header(storage.header.block_len as usize);
        if init_storage_result.is_err() {
            return Err(Error {
                code: 2,
                message: "Could not init storage".to_string(),
            });
        }
        Ok(storage)
    }
    /// Set storage header in storage file
    /// - Write storage header to file
    /// - NOTE: This can only be used once when creating a new storage file
    fn set_storage_header(&mut self, block_len: usize) -> Result<usize, Error> {
        use std::io::prelude::*;
        let file = &mut self.file_writer;
        let header_bytes = u32_to_bytes(block_len as u32);
        // - seek writer pointer to beginning of file
        // - write storage header
        let ptr_seek_result = file.seek(std::io::SeekFrom::Start(0));
        if ptr_seek_result.is_err() {
            return Err(Error {
                code: 3,
                message: "Could not seek file pointer".to_string(),
            });
        }
        self.write_pointer = ptr_seek_result.unwrap();
        let write_result = file.write(&header_bytes);
        if write_result.is_err() {
            return Err(Error {
                code: 2,
                message: "Could not write to file".to_string(),
            });
        }
        // - verify write operation was successful
        let write_size = write_result.unwrap();
        if write_size != header_bytes.len() {
            return Err(Error {
                code: 2,
                message: "Could not write all header bytes to file".to_string(),
            });
        }
        self.header.block_len = block_len as u32;
        self.write_pointer += write_size as u64;
        Ok(write_size)
    }
    pub fn read_block(&mut self, block_index: usize) -> Result<(usize, Vec<u8>), Error> {
        use std::io::prelude::*;
        let block_size = self.header.block_len;
        let block_offset: usize = STORAGE_HEADER_SIZE as usize
            + block_index as usize * (BLOCK_HEADER_SIZE as usize + block_size as usize);
        // - seek reader to block offset
        let seek_result = self
            .file_reader
            .seek(std::io::SeekFrom::Start(block_offset as u64));
        if seek_result.is_err() {
            return Err(Error {
                code: 3,
                message: "Could not seek to block offset".to_string(),
            });
        }
        // verify seek operation was successful
        let seek_position = seek_result.unwrap();
        if seek_position != block_offset as u64 {
            return Err(Error {
                code: 3,
                message: "Could not seek to block offset".to_string(),
            });
        }
        // - read block data length from inital 4 bytes
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
        // - read block data to vec
        let mut block_data = vec![0u8; block_header.block_data_size as usize];
        let read_result = self.file_reader.read(&mut block_data[..]);
        if read_result.is_err() {
            return Err(Error {
                code: 4,
                message: "Could not read from file".to_string(),
            });
        }
        let read_size = read_result.unwrap() as u32;
        self.read_pointer += read_size as u64;
        // - return read_pointer and block_data
        Ok((self.read_pointer as usize, block_data))
    }
    pub fn write_block(&mut self, block_index: usize, data: Vec<u8>) -> Result<usize, Error> {
        use std::io::prelude::*;
        let block_size = self.header.block_len;
        let block_offset = STORAGE_HEADER_SIZE as usize
            + block_index as usize * (BLOCK_HEADER_SIZE as usize + block_size as usize);
        // - seek writer to block offset
        let seek_result = self
            .file_writer
            .seek(std::io::SeekFrom::Start(block_offset as u64));
        if seek_result.is_err() {
            return Err(Error {
                code: 5,
                message: "Could not seek to block offset".to_string(),
            });
        }
        // - verify seek operation was successful
        let seek_position = seek_result.unwrap();
        if seek_position != block_offset as u64 {
            return Err(Error {
                code: 5,
                message: "Could not seek to block offset".to_string(),
            });
        }
        // - Write Block Header
        // -- write block header length to inital 4 bytes
        let block_data_size_bytes = u32_to_bytes(data.len() as u32);
        let write_result = self.file_writer.write(&block_data_size_bytes);
        if write_result.is_err() {
            return Err(Error {
                code: 6,
                message: "Could not write to file".to_string(),
            });
        }
        let write_size = write_result.unwrap();
        // -- verify write operation was successful
        if write_size != data.len() {
            return Err(Error {
                code: 8,
                message: "Could not write all data to file".to_string(),
            });
        }
        // - Write Block Data
        // -- write block data to file
        let write_result = self.file_writer.write(&data[..]);
        if write_result.is_err() {
            return Err(Error {
                code: 7,
                message: "Could not write to file".to_string(),
            });
        }
        let write_size = write_result.unwrap();
        // -- verify write operation was successful
        if write_size != data.len() {
            return Err(Error {
                code: 9,
                message: "Could not write all data to file".to_string(),
            });
        }
        let write_pointer: usize =
            block_offset as usize + BLOCK_HEADER_SIZE as usize + write_size as usize;
        // update block header
        let block_index = block_index as u32;
        self.free_blocks.remove(&block_index);
        // return write pointer
        Ok(write_pointer)
    }
}