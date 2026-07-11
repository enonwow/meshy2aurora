pub const FILE_HEADER_SIZE: usize = 0x0c;
pub const MODEL_HEADER_SIZE: usize = 0xe8;
pub const NODE_HEADER_SIZE: usize = 0x70;
pub const ROOT_NODE_OFFSET: u32 = MODEL_HEADER_SIZE as u32;
pub const ROOT_NODE_ABSOLUTE: usize = FILE_HEADER_SIZE + ROOT_NODE_OFFSET as usize;

pub fn build_minimal_binary_mdl() -> Vec<u8> {
    let core_size = MODEL_HEADER_SIZE + NODE_HEADER_SIZE;
    let mut bytes = vec![0_u8; FILE_HEADER_SIZE + core_size];
    write_file_and_model_header(&mut bytes, core_size, 1);
    write_node(&mut bytes, ROOT_NODE_OFFSET, 0, "root", 0, 0, 0);
    bytes
}

pub fn build_two_node_binary_mdl() -> Vec<u8> {
    let child_offset = ROOT_NODE_OFFSET + NODE_HEADER_SIZE as u32;
    let children_array_offset = child_offset + NODE_HEADER_SIZE as u32;
    let core_size = children_array_offset as usize + 4;
    let mut bytes = vec![0_u8; FILE_HEADER_SIZE + core_size];
    write_file_and_model_header(&mut bytes, core_size, 2);
    write_node(
        &mut bytes,
        ROOT_NODE_OFFSET,
        0,
        "root",
        0,
        children_array_offset,
        1,
    );
    write_node(&mut bytes, child_offset, 1, "child", ROOT_NODE_OFFSET, 0, 0);
    write_u32(
        &mut bytes,
        FILE_HEADER_SIZE + children_array_offset as usize,
        child_offset,
    );
    bytes
}

pub fn make_root_cycle(bytes: &mut Vec<u8>) {
    let array_offset = (bytes.len() - FILE_HEADER_SIZE) as u32;
    bytes.extend_from_slice(&ROOT_NODE_OFFSET.to_le_bytes());
    write_u32(bytes, 4, array_offset as usize as u32 + 4);
    write_u32(bytes, ROOT_NODE_ABSOLUTE + 0x48, array_offset);
    write_u32(bytes, ROOT_NODE_ABSOLUTE + 0x4c, 1);
    write_u32(bytes, ROOT_NODE_ABSOLUTE + 0x50, 1);
}

pub fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_file_and_model_header(bytes: &mut [u8], core_size: usize, node_count: u32) {
    write_u32(bytes, 0, 0);
    write_u32(bytes, 4, core_size as u32);
    write_u32(bytes, 8, 0);

    write_c_string(bytes, FILE_HEADER_SIZE + 0x08, 64, "m2a_minimal");
    write_u32(bytes, FILE_HEADER_SIZE + 0x48, ROOT_NODE_OFFSET);
    write_u32(bytes, FILE_HEADER_SIZE + 0x4c, node_count);
}

fn write_node(
    bytes: &mut [u8],
    offset: u32,
    number: u32,
    name: &str,
    parent: u32,
    children_offset: u32,
    children_count: u32,
) {
    let absolute = FILE_HEADER_SIZE + offset as usize;
    write_u32(bytes, absolute + 0x1c, number);
    write_c_string(bytes, absolute + 0x20, 32, name);
    write_u32(bytes, absolute + 0x44, parent);
    write_u32(bytes, absolute + 0x48, children_offset);
    write_u32(bytes, absolute + 0x4c, children_count);
    write_u32(bytes, absolute + 0x50, children_count);
    write_u32(bytes, absolute + 0x6c, 1);
}

fn write_c_string(bytes: &mut [u8], offset: usize, capacity: usize, value: &str) {
    assert!(value.len() < capacity);
    bytes[offset..offset + value.len()].copy_from_slice(value.as_bytes());
}
