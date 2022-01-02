pub mod storage;
use std::sync::mpsc::{Receiver, Sender};
use storage::{error::Error, BlockIndex, Storage};

type ReadRequestChanRes = Result<Vec<u8>, Error>;
type ReadRequest = (
    Vec<BlockIndex>, /* block indexes to read from */
    Sender<ReadRequestChanRes>,
    Receiver<ReadRequestChanRes>,
);

type WriteRequestChanRes = Result<Vec<BlockIndex>, Error>;
type WriteRequest = (
    Vec<u8>, /* data payload */
    Sender<WriteRequestChanRes>,
    Receiver<WriteRequestChanRes>,
);

type DeleteRequestChanRes = Result<(), Error>;
type DeleteRequest = (
    (Vec<BlockIndex>, bool), /* (block_indexes_to_delete, hard_delete_T_F) */
    Sender<DeleteRequestChanRes>,
    Receiver<DeleteRequestChanRes>,
);

pub type IORequest = (
    Option<ReadRequest>,
    Option<WriteRequest>,
    Option<DeleteRequest>,
);

use std::collections::LinkedList;
pub struct Engine {
    storage: Storage,
    request_queue: LinkedList<IORequest>,
}

impl Engine {
    pub fn new(storage: Storage) -> Self {
        Engine {
            storage: storage,
            request_queue: LinkedList::new(),
        }
    }
    pub fn io_cycle(engine: &mut Engine) {
        let mut read_requests: Vec<&ReadRequest> = Vec::new();
        let mut write_requests: Vec<&WriteRequest> = Vec::new();
        let mut delete_requests: Vec<&DeleteRequest> = Vec::new();
        for request in &engine.request_queue {
            match request {
                (Some(read_request), _, _) => read_requests.push(read_request),
                (_, Some(write_request), _) => write_requests.push(write_request),
                (_, _, Some(delete_request)) => delete_requests.push(delete_request),
                (None, None, None) => panic!("Invalid request"),
            }
        }
        // - Atomic Lock
        // - Serve Reads
        for readRequest in read_requests {
            let (indexes, sender, receiver) = readRequest;
            let mut data: Vec<u8> = Vec::new();
            // indexes must be pre-sorted
            for index_iter in indexes {
                let index = *index_iter;
                let read_result = engine.storage.read_block(index);
                if read_result.is_err() {
                    sender.send(Err(read_result.err().unwrap())).unwrap();
                    return;
                }
                let (read_ptr, read_data) = read_result.unwrap();
                data.copy_from_slice(&read_data);
            }
            sender.send(Ok(data)).unwrap();
        }
        // - Atomic Lock
        // - Allocate blocks for writes
        // - Write to allocated blocks
        for writeRequest in write_requests {
            let (data, sender, receiver) = writeRequest;
            let indexes: Vec<BlockIndex> = engine
                .storage
                .search_block_allocation_indexes(data.len() as BlockIndex);
            let mut data_write_ptr = 0 as usize;
            for index in indexes.clone() {
                let data_chunk =
                    &data[data_write_ptr..(data_write_ptr + engine.storage.block_len() as usize)];
                let write_result = engine.storage.write_block(index, data_chunk);
                if write_result.is_err() {
                    sender.send(Err(write_result.err().unwrap())).unwrap();
                    return;
                }
                data_write_ptr += data_chunk.len();
            }
            sender.send(Ok(indexes)).unwrap();
        }
        // - Atomic Lock
        // - Serve Delete requests
        for deleteRequest in delete_requests {
            let ((indexes, hard_delete), sender, receiver) = deleteRequest;
            for index in indexes {
                let delete_result = engine.storage.delete_block(*index, *hard_delete);
                if delete_result.is_err() {
                    sender.send(Err(delete_result.err().unwrap())).unwrap();
                    return;
                }
            }
            sender.send(Ok(())).unwrap();
        }
        // - Atomic Lock
    }
    pub fn append_request(&mut self, request: IORequest) {
        self.request_queue.push_back(request);
    }
    pub fn clear_requests(&mut self) {
        self.request_queue.clear();
    }
}
