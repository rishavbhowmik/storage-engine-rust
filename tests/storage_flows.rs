use se1::storage::Storage;

fn read_full_file(file_name: &str) -> Vec<u8> {
    use std::fs::read;
    use std::path::Path;
    let read_result = read(Path::new(file_name));
    match read_result {
        Ok(data) => data,
        Err(e) => panic!("{:?}", e),
    }
}

#[test]
fn storage_open_new_file() {
    fn fetch_state(state_file: &str) -> Vec<u8> {
        use std::path::PathBuf;
        let path: PathBuf = ["tests/samples/storage_open_new_file_states", state_file].iter().collect();
        read_full_file(path.to_str().unwrap())
    }
    let tmp_file_path = "./tmp/storage_open_new_file.hex";
    // create new storage
    let storage_result = Storage::new(String::from(tmp_file_path), 8);
    assert_eq!(storage_result.is_ok(), true);
    let mut storage = storage_result.unwrap();
    let expected = fetch_state("on_create.hex");
    let actual = read_full_file(tmp_file_path);
    assert_eq!(expected, actual);
    // write to block 0
    let block_0_data = vec![1 as u8, 2 as u8, 3 as u8, 4 as u8, 5 as u8, 6 as u8, 7 as u8, 8 as u8];
    let result = storage.write_block(0, &block_0_data);
    assert_eq!(result.is_ok(), true);
    let write_ptr = result.unwrap();
    assert_eq!(write_ptr, 16); // 4 + (4 + 8) * 0 + 4 + 8
    let expected = fetch_state("on_write_block_0.hex");
    let actual = read_full_file(tmp_file_path);
    assert_eq!(expected, actual);
    // write to block 1
    let block_1_data = vec![9 as u8, 10 as u8, 11 as u8, 12 as u8, 13 as u8, 14 as u8, 15 as u8, 16 as u8];
    let result = storage.write_block(1, &block_1_data);
    assert_eq!(result.is_ok(), true);
    let write_ptr = result.unwrap();
    assert_eq!(write_ptr, 28); // 4 + (4 + 8) * 1 + 4 + 8
    let expected = fetch_state("on_write_block_1.hex");
    let actual = read_full_file(tmp_file_path);
    assert_eq!(expected, actual);
    // write to block 2
    let block_2_data = vec![17 as u8, 18 as u8, 19 as u8, 20 as u8];
    let result = storage.write_block(2, &block_2_data);
    assert_eq!(result.is_ok(), true);
    let write_ptr = result.unwrap();
    assert_eq!(write_ptr, 36); // 4 + (4 + 8) * 2 + 4 + 4
    let expected = fetch_state("on_write_block_2.hex");
    let actual = read_full_file(tmp_file_path);
    assert_eq!(expected, actual);
    // read from block 2
    let result = storage.read_block(2);
    assert_eq!(result.is_ok(), true);
    let (read_ptr, actual_data) = result.unwrap();
    assert_eq!(read_ptr, 36); // 4 + (4 + 8) * 2 + 4 + 4
    assert_eq!(actual_data, block_2_data);
    // read from block 1
    let result = storage.read_block(1);
    assert_eq!(result.is_ok(), true);
    let (read_ptr, actual_data) = result.unwrap();
    assert_eq!(read_ptr, 28); // 4 + (4 + 8) * 1 + 4 + 8
    assert_eq!(actual_data, block_1_data);
    // read from block 0
    let result = storage.read_block(0);
    assert_eq!(result.is_ok(), true);
    let (read_ptr, actual_data) = result.unwrap();
    assert_eq!(read_ptr, 16); // 4 + (4 + 8) * 0 + 4 + 8
    assert_eq!(actual_data, block_0_data);
    // read from block 3
    let result = storage.read_block(3);
    assert_eq!(result.is_ok(), true);
    let (read_ptr, actual_data) = result.unwrap();
    assert_eq!(read_ptr, 16); // no change
    assert_eq!(actual_data.len(), 0); // no data
    // soft delete_block 0
    let result = storage.delete_block(0, false);
    assert_eq!(result.is_ok(), true);
    let write_ptr = result.unwrap();
    assert_eq!(write_ptr, 8); // 4 + (4 + 8) * 0 + 4 + 0
    let expected = fetch_state("on_soft_delete_block_0.hex");
    let actual = read_full_file(tmp_file_path);
    assert_eq!(expected, actual);
    // hard delete_block 0
    let result = storage.delete_block(0, true);
    assert_eq!(result.is_ok(), true);
    let write_ptr = result.unwrap();
    assert_eq!(write_ptr, 16); // 4 + (4 + 8) * 0 + 4 + 8
    let expected = fetch_state("on_hard_delete_block_0.hex");
    let actual = read_full_file(tmp_file_path);
    assert_eq!(expected, actual);
    // soft delete_block 1
    let result = storage.delete_block(1, false);
    assert_eq!(result.is_ok(), true);
    let write_ptr = result.unwrap();
    assert_eq!(write_ptr, 20); // 4 + (4 + 8) * 1 + 4 + 0
    let expected = fetch_state("on_soft_delete_block_1.hex");
    let actual = read_full_file(tmp_file_path);
    assert_eq!(expected, actual);
    // hard delete_block 2
    let result = storage.delete_block(2, true);
    assert_eq!(result.is_ok(), true);
    let write_ptr = result.unwrap();
    assert_eq!(write_ptr, 40); // 4 + (4 + 8) * 2 + 4 + 8
    let expected = fetch_state("on_hard_delete_block_2.hex");
    let actual = read_full_file(tmp_file_path);
    assert_eq!(expected, actual);
}