mod error;
use error::Error;
mod util;
use util::*;

/// Main Header for storage file
/// - Stores constant capacity of each block as 4 bytes unsied integer as little endian
struct StorageHeader {
    block_len: u32,
}

const STORAGE_HEADER_SIZE: usize = std::mem::size_of::<StorageHeader>();

impl StorageHeader {
    fn new(block_len: u32) -> Self {
        StorageHeader { block_len }
    }
    fn from_bytes(bytes: &[u8; STORAGE_HEADER_SIZE]) -> StorageHeader {
        let block_len = bytes_to_u32(bytes);
        StorageHeader { block_len }
    }
    fn to_bytes(&self) -> [u8; STORAGE_HEADER_SIZE] {
        u32_to_bytes(self.block_len)
    }
}

/// Header of each block
/// - Stores size of data stored in the block as 4 bytes unsied integer as little endian
struct BlockHeader {
    block_data_size: u32,
}

const BLOCK_HEADER_SIZE: usize = std::mem::size_of::<BlockHeader>();

impl BlockHeader {
    fn new(block_data_size: u32) -> BlockHeader {
        BlockHeader {
            block_data_size: block_data_size,
        }
    }
    fn from_bytes(bytes: &[u8; BLOCK_HEADER_SIZE]) -> BlockHeader {
        let block_data_size = bytes_to_u32(bytes);
        BlockHeader {
            block_data_size: block_data_size,
        }
    }
    fn to_bytes(&self) -> [u8; BLOCK_HEADER_SIZE] {
        u32_to_bytes(self.block_data_size)
    }
}

use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};

pub struct Storage {
    header: StorageHeader,
    /// Map of empty blocks in the storage file
    free_blocks: BTreeSet<u32>,
    /// Number of blocks in the storage file (used or free)
    end_block_count: u32,
    /// File object for writing
    file_writer: File,
    /// Index of last written byte in the file
    write_pointer: u64,
    /// File object for reading
    file_reader: File,
    /// Index of last read byte in the file
    read_pointer: u64,
}

impl Storage {
    fn open_file_writer(file_path: &String, truncate: bool) -> Result<(File, u64), Error> {
        let file_path_clone = file_path.clone();
        let file_writer_result = OpenOptions::new()
            .write(true)
            .truncate(truncate)
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
        let file_writer = Storage::open_file_writer(&file_path, true);
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
            header: StorageHeader::new(block_len as u32),
            free_blocks: BTreeSet::new(),
            end_block_count: 0,
            file_writer,
            write_pointer,
            file_reader,
            read_pointer,
        };
        if storage.set_storage_header().is_err() {
            return Err(Error {
                code: 2,
                message: "Could not init storage".to_string(),
            });
        }
        Ok(storage)
    }
    /// Open existing storage file
    /// - Loads storage header
    /// - Loads free blocks Set
    pub fn open(file_path: String) -> Result<Storage, Error> {
        let file_writer = Storage::open_file_writer(&file_path, false);
        if file_writer.is_err() {
            return Err(file_writer.unwrap_err());
        }
        let (file_writer, write_pointer) = file_writer.unwrap();
        let file_reader = Storage::open_file_reader(&file_path);
        if file_reader.is_err() {
            return Err(file_reader.unwrap_err());
        }
        let (file_reader, read_pointer) = file_reader.unwrap();

        // - init storage object
        let mut storage = Storage {
            header: StorageHeader::new(0),
            free_blocks: BTreeSet::new(),
            end_block_count: 0,
            file_writer,
            write_pointer,
            file_reader,
            read_pointer,
        };
        // - read and update storage header from file
        if storage.get_storage_header().is_err() {
            return Err(Error {
                code: 2,
                message: "Could not init storage".to_string(),
            });
        }
        // - read file and count
        // -- total blocks - update self.end_block_count
        // -- free blocks - update self.free_blocks
        let blocks_status_result = storage.read_storage_block_headers();
        if blocks_status_result.is_err() {
            return Err(blocks_status_result.unwrap_err());
        }
        Ok(storage)
    }

    // # File IO Functions

    /// Set storage header in storage file
    /// - Write storage header to file
    /// - NOTE: This can only be used once when creating a new storage file
    fn set_storage_header(&mut self) -> Result<usize, Error> {
        use std::io::prelude::*;
        let file = &mut self.file_writer;
        // Write storage header to file
        let header_bytes = self.header.to_bytes();
        // -- seek writer pointer to beginning of file
        let ptr_seek_result = file.seek(std::io::SeekFrom::Start(0));
        if ptr_seek_result.is_err() {
            return Err(Error {
                code: 3,
                message: "Could not seek file pointer".to_string(),
            });
        }
        // -- write storage header
        self.write_pointer = ptr_seek_result.unwrap();
        let write_result = file.write(&header_bytes);
        if write_result.is_err() {
            return Err(Error {
                code: 2,
                message: "Could not write to file".to_string(),
            });
        }
        // -- verify write operation was successful
        let write_size = write_result.unwrap();
        if write_size != STORAGE_HEADER_SIZE as usize {
            return Err(Error {
                code: 2,
                message: "Could not write all header bytes to file".to_string(),
            });
        }
        self.write_pointer += write_size as u64;
        Ok(write_size)
    }
    /// Get storage header from storage file
    /// - Read storage header from file
    /// - update storage header in object
    fn get_storage_header(&mut self) -> Result<usize, Error> {
        use std::io::prelude::*;
        let file = &mut self.file_reader;
        // - Read storage header from file
        // -- seek reader pointer to beginning of file
        let ptr_seek_result = file.seek(std::io::SeekFrom::Start(0));
        if ptr_seek_result.is_err() {
            return Err(Error {
                code: 3,
                message: "Could not seek file pointer".to_string(),
            });
        }
        // -- read storage header
        let mut header_bytes = [0u8; STORAGE_HEADER_SIZE];
        self.read_pointer = ptr_seek_result.unwrap();
        let read_result = file.read(&mut header_bytes);
        if read_result.is_err() {
            return Err(Error {
                code: 2,
                message: "Could not read from file".to_string(),
            });
        }
        // -- verify read operation was successful
        let read_size = read_result.unwrap();
        if read_size != STORAGE_HEADER_SIZE as usize {
            return Err(Error {
                code: 2,
                message: "Could not read all header bytes from file".to_string(),
            });
        }
        // -- update read pointer
        self.read_pointer += read_size as u64;
        // - parse storage header
        let storage_header = StorageHeader::from_bytes(&header_bytes);
        // - copy storage header to storage object
        self.header = storage_header;
        // - return read pointer
        Ok(read_size)
    }
    /// Count number of blocks in storage file
    /// -- total blocks - update self.end_block_count
    /// -- free blocks - update self.free_blocks
    fn read_storage_block_headers(&mut self) -> Result<usize, Error> {
        use std::io::prelude::*;
        let file = &mut self.file_reader;
        // - seek reader pointer to end of file
        let ptr_seek_result = file.seek(std::io::SeekFrom::Start(0));
        if ptr_seek_result.is_err() {
            return Err(Error {
                code: 3,
                message: "Could not seek file pointer".to_string(),
            });
        }
        // - update read pointer
        self.read_pointer = ptr_seek_result.unwrap();
        // - read file and count
        // -- total blocks - update self.end_block_count
        // -- free blocks - update self.free_blocks
        let mut free_blocks = BTreeSet::new();
        // -- seek reader pointer to end of STORAGE_HEADER_SIZE
        let ptr_seek_result = file.seek(std::io::SeekFrom::Start(STORAGE_HEADER_SIZE as u64));
        if ptr_seek_result.is_err() {
            return Err(Error {
                code: 3,
                message: "Could not seek file pointer".to_string(),
            });
        }
        // -- traverse all blocks in file, untill end of file
        let mut block_index = 0;
        loop {
            // - read block header
            let mut block_header_bytes = [0u8; BLOCK_HEADER_SIZE];
            let read_result = file.read(&mut block_header_bytes);
            if read_result.is_err() {
                return Err(Error {
                    code: 2,
                    message: "Could not read from file".to_string(),
                });
            }
            // -- check end of file
            // -- verify read operation was successful
            let read_size = read_result.unwrap();
            if read_size == 0 {
                // end of file reached
                break;
            }
            if read_size != BLOCK_HEADER_SIZE as usize {
                return Err(Error {
                    code: 2,
                    message: "Could not read all header bytes from file".to_string(),
                });
            }
            // -- update read pointer
            self.read_pointer += read_size as u64;
            // -- parse block header
            let block_header = BlockHeader::from_bytes(&block_header_bytes);
            // - check if block is free
            if block_header.block_data_size == 0 {
                // -- add block to free blocks
                free_blocks.insert(block_index);
            }
            // -- increment block index
            block_index += 1;
            // - seek reader pointer to end of block
            let ptr_seek_result =
                file.seek(std::io::SeekFrom::Current(self.header.block_len as i64));
            if ptr_seek_result.is_err() {
                return Err(Error {
                    code: 3,
                    message: "Could not seek file pointer".to_string(),
                });
            }
            let ptr_seek_result = ptr_seek_result.unwrap();
            self.read_pointer = ptr_seek_result;
            // -- verify seek operation was successful
            if ptr_seek_result != self.read_pointer {
                // end of file reached
                break;
            }
        }
        // - update end block count
        self.end_block_count = block_index;
        // - update free blocks
        self.free_blocks = free_blocks;
        // - return
        Ok(self.read_pointer as usize)
    }
    /// check if block is within storage file, without reading it from file (in memory)
    fn block_exists(&mut self, block_index: u32) -> bool {
        block_index < self.end_block_count
    }
    /// Check if block is empty, without reading it from file (in memory)
    fn is_empty_block(&mut self, block_index: usize) -> bool {
        let block_index = block_index as u32;
        if self.block_exists(block_index) {
            if self.free_blocks.contains(&block_index) {
                return true;
            } else {
                return false;
            }
        } else {
            return true;
        }
    }
    pub fn read_block(&mut self, block_index: usize) -> Result<(usize, Vec<u8>), Error> {
        if self.is_empty_block(block_index) {
            // return current read_pointer and empty vector
            return Ok((self.read_pointer as usize, Vec::new()));
        }
        use std::io::prelude::*;
        let block_length = self.header.block_len;
        let block_offset: usize = STORAGE_HEADER_SIZE as usize
            + block_index as usize * (BLOCK_HEADER_SIZE as usize + block_length as usize);
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
        let block_length = self.header.block_len;
        let block_offset = STORAGE_HEADER_SIZE as usize
            + block_index as usize * (BLOCK_HEADER_SIZE as usize + block_length as usize);
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
        // -- verify seek operation was successful
        let seek_position = seek_result.unwrap();
        if seek_position != block_offset as u64 {
            return Err(Error {
                code: 5,
                message: "Could not seek to block offset".to_string(),
            });
        }
        // - Write Block Header
        // -- write block header to inital BLOCK_HEADER_SIZE bytes
        let block_header = BlockHeader::new(data.len() as u32);
        let write_result = self.file_writer.write(&block_header.to_bytes());
        if write_result.is_err() {
            return Err(Error {
                code: 6,
                message: "Could not write to file".to_string(),
            });
        }
        let write_size = write_result.unwrap();
        // -- verify write operation was successful
        if write_size != BLOCK_HEADER_SIZE {
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
        // - update write ptr
        self.write_pointer = block_offset as u64 + BLOCK_HEADER_SIZE as u64 + write_size as u64;
        // - update free_blocks map
        let block_index = block_index as u32;
        self.free_blocks.remove(&block_index);
        // - update max_block_index
        if block_index >= self.end_block_count {
            self.end_block_count = block_index + 1;
        }
        // return write pointer
        Ok(self.write_pointer as usize)
    }
    pub fn delete_block(&mut self, block_index: usize, hard_delete: bool) -> Result<usize, Error> {
        let block_index = block_index as u32;
        if !self.block_exists(block_index as u32) {
            return Ok(self.write_pointer as usize);
        } else if hard_delete == false && self.free_blocks.contains(&block_index) {
            return Ok(self.write_pointer as usize);
        }
        use std::io::prelude::*;
        let block_length = self.header.block_len;
        let block_offset = STORAGE_HEADER_SIZE as usize
            + block_index as usize * (BLOCK_HEADER_SIZE as usize + block_length as usize);
        // - seek writer to block offset
        let seek_result = self
            .file_writer
            .seek(std::io::SeekFrom::Start(block_offset as u64));
        if seek_result.is_err() {
            return Err(Error {
                code: 10,
                message: "Could not seek to block offset".to_string(),
            });
        }
        // -- verify seek operation was successful
        let seek_position = seek_result.unwrap();
        if seek_position != block_offset as u64 {
            return Err(Error {
                code: 10,
                message: "Could not seek to block offset".to_string(),
            });
        }
        // - Write Block Header
        // -- write block header to inital BLOCK_HEADER_SIZE bytes
        let block_header = BlockHeader::new(0);
        let write_result = self.file_writer.write(&block_header.to_bytes());
        if write_result.is_err() {
            return Err(Error {
                code: 11,
                message: "Could not write to file".to_string(),
            });
        }
        let write_size = write_result.unwrap();
        self.write_pointer = block_offset as u64 + BLOCK_HEADER_SIZE as u64 + write_size as u64;
        // -- verify write operation was successful
        if write_size != BLOCK_HEADER_SIZE {
            return Err(Error {
                code: 12,
                message: "Could not write all data to file".to_string(),
            });
        }
        // - hard delete block
        if hard_delete == true {
            // post successful block header write, writer pointer must be at data offset
            // - overwrite full block with zeros
            let block_data_of_zeros = vec![0u8; block_length as usize];
            let write_result = self.file_writer.write(&block_data_of_zeros[..]);
            if write_result.is_err() {
                return Err(Error {
                    code: 13,
                    message: "Could not write to file".to_string(),
                });
            }
            let write_size = write_result.unwrap();
            // -- verify write operation was successful
            if write_size != block_length as usize {
                return Err(Error {
                    code: 14,
                    message: "Could not write all data to file".to_string(),
                });
            }
            // -- increment write pointer
            self.write_pointer += write_size as u64;
        }
        // update free_blocks map
        self.free_blocks.insert(block_index);
        // return write pointer
        Ok(self.write_pointer as usize)
    }
}
