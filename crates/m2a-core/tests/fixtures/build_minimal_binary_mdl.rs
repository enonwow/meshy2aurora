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

pub fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub fn write_i16(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

pub fn build_deep_binary_mdl() -> Vec<u8> {
    let root = ROOT_NODE_OFFSET;
    let mut core = vec![0_u8; root as usize + 0x270];
    write_model_core(&mut core, "m2a_deep", root, 2);
    write_node_core(&mut core, root, 0, "root", 0x21);
    write_mesh_raw_headers(&mut core, root);

    let face = append_zeros(&mut core, 0x20);
    write_array_core(&mut core, root as usize + 0x78, face, 1, 1);
    write_face_core(&mut core, face);

    let keys = append_zeros(&mut core, 5 * 0x0c);
    let data = append_zeros(&mut core, 13 * 4);
    write_array_core(&mut core, root as usize + 0x54, keys, 5, 5);
    write_array_core(&mut core, root as usize + 0x60, data, 13, 13);
    for (index, (kind, data_index, columns)) in
        [(8, 1, 3), (20, 4, 4), (36, 8, 1), (100, 9, 3), (128, 12, 1)]
            .into_iter()
            .enumerate()
    {
        write_controller_core(
            &mut core,
            keys as usize + index * 0x0c,
            kind,
            1,
            0,
            data_index,
            columns,
        );
    }
    for (index, value) in [
        0.0_f32, 1.0, 2.0, 3.0, 1.0, 0.0, 0.0, 0.0, 1.25, 0.2, 0.3, 0.4, 0.75,
    ]
    .into_iter()
    .enumerate()
    {
        write_f32_core(&mut core, data as usize + index * 4, value);
    }

    let animation_pointers = append_zeros(&mut core, 8);
    let animation1 = append_zeros(&mut core, 0xc4);
    let animation1_root = append_zeros(&mut core, 0x70);
    let animation1_event = append_zeros(&mut core, 0x24);
    let animation1_key = append_zeros(&mut core, 0x0c);
    let animation1_data = append_zeros(&mut core, 4 * 4);
    let animation2 = append_zeros(&mut core, 0xc4);
    let animation2_root = append_zeros(&mut core, 0x70);
    let animation2_event = append_zeros(&mut core, 0x24);
    write_array_core(&mut core, 0x78, animation_pointers, 2, 2);
    write_u32_core(&mut core, animation_pointers as usize, animation1);
    write_u32_core(&mut core, animation_pointers as usize + 4, animation2);
    write_animation_core(
        &mut core,
        animation1,
        "walk",
        animation1_root,
        animation1_event,
        "footstep",
        1.5,
    );
    write_node_core(&mut core, animation1_root, 0, "walk_root", 0x01);
    write_array_core(
        &mut core,
        animation1_root as usize + 0x54,
        animation1_key,
        1,
        1,
    );
    write_array_core(
        &mut core,
        animation1_root as usize + 0x60,
        animation1_data,
        4,
        4,
    );
    write_controller_core(&mut core, animation1_key as usize, 8, 1, 0, 1, 3);
    for (index, value) in [0.0_f32, 9.0, 8.0, 7.0].into_iter().enumerate() {
        write_f32_core(&mut core, animation1_data as usize + index * 4, value);
    }
    write_animation_core(
        &mut core,
        animation2,
        "idle",
        animation2_root,
        animation2_event,
        "loop",
        2.0,
    );
    write_node_core(&mut core, animation2_root, 0, "idle_root", 0x01);

    let raw = build_mesh_raw(false, 17);
    finish_binary(core, raw)
}

pub fn build_skin_binary_mdl(extended64: bool) -> Vec<u8> {
    let root = ROOT_NODE_OFFSET;
    let profile_size = if extended64 { 0x330 } else { 0x2d4 };
    let inline_count = if extended64 { 64 } else { 17 };
    let mut core = vec![0_u8; root as usize + profile_size];
    write_model_core(
        &mut core,
        if extended64 {
            "m2a_skin64"
        } else {
            "m2a_skin17"
        },
        root,
        1,
    );
    write_node_core(&mut core, root, 0, "skin_root", 0x61);
    write_mesh_raw_headers(&mut core, root);
    let map = append_zeros(&mut core, 3 * 2);
    let quaternions = append_zeros(&mut core, 16);
    let translations = append_zeros(&mut core, 12);
    let constants = append_zeros(&mut core, 4);
    let face = append_zeros(&mut core, 0x20);
    write_array_core(&mut core, root as usize + 0x78, face, 1, 1);
    write_face_core(&mut core, face);
    write_i32_core(&mut core, root as usize + 0x284, map as i32);
    write_i32_core(&mut core, root as usize + 0x288, 3);
    write_array_core(&mut core, root as usize + 0x28c, quaternions, 1, 1);
    write_array_core(&mut core, root as usize + 0x298, translations, 1, 1);
    write_array_core(&mut core, root as usize + 0x2a4, constants, 1, 1);
    for index in 0..inline_count {
        write_i16_core(&mut core, root as usize + 0x2b0 + index * 2, index as i16);
    }
    for (index, value) in [0_i16, 1, 2].into_iter().enumerate() {
        write_i16_core(&mut core, map as usize + index * 2, value);
    }
    write_f32_core(&mut core, quaternions as usize, 1.0);
    write_f32_core(&mut core, translations as usize, 4.0);
    write_i16_core(&mut core, constants as usize, 7);
    write_i16_core(&mut core, constants as usize + 2, 8);
    write_i32_core(&mut core, root as usize + 0x27c, 96);
    write_i32_core(&mut core, root as usize + 0x280, 144);
    let raw = build_mesh_raw(true, inline_count);
    finish_binary(core, raw)
}

pub fn make_animation_root_cycle(bytes: &mut Vec<u8>) {
    let (animation1, animation1_root, _) = animation_offsets(bytes);
    let cycle_pointer = append_core_pointer(bytes, animation1_root);
    write_u32(
        bytes,
        FILE_HEADER_SIZE + animation1_root as usize + 0x48,
        cycle_pointer,
    );
    write_u32(bytes, FILE_HEADER_SIZE + animation1_root as usize + 0x4c, 1);
    write_u32(bytes, FILE_HEADER_SIZE + animation1_root as usize + 0x50, 1);
    write_u32(bytes, FILE_HEADER_SIZE + animation1 as usize + 0x4c, 2);
}

pub fn make_animation_declared_too_small(bytes: &mut Vec<u8>) {
    let (animation1, animation1_root, animation2_root) = animation_offsets(bytes);
    let child_pointer = append_core_pointer(bytes, animation2_root);
    write_u32(
        bytes,
        FILE_HEADER_SIZE + animation1_root as usize + 0x48,
        child_pointer,
    );
    write_u32(bytes, FILE_HEADER_SIZE + animation1_root as usize + 0x4c, 1);
    write_u32(bytes, FILE_HEADER_SIZE + animation1_root as usize + 0x50, 1);
    write_u32(bytes, FILE_HEADER_SIZE + animation1 as usize + 0x4c, 1);
}

fn animation_offsets(bytes: &[u8]) -> (u32, u32, u32) {
    let pointer_list = read_u32_bytes(bytes, FILE_HEADER_SIZE + 0x78);
    let animation1 = read_u32_bytes(bytes, FILE_HEADER_SIZE + pointer_list as usize);
    let animation2 = read_u32_bytes(bytes, FILE_HEADER_SIZE + pointer_list as usize + 4);
    let animation1_root = read_u32_bytes(bytes, FILE_HEADER_SIZE + animation1 as usize + 0x48);
    let animation2_root = read_u32_bytes(bytes, FILE_HEADER_SIZE + animation2 as usize + 0x48);
    (animation1, animation1_root, animation2_root)
}

fn append_core_pointer(bytes: &mut Vec<u8>, value: u32) -> u32 {
    let core_length = read_u32_bytes(bytes, 4);
    let raw_absolute = FILE_HEADER_SIZE + core_length as usize;
    bytes.splice(raw_absolute..raw_absolute, value.to_le_bytes());
    write_u32(bytes, 4, core_length + 4);
    core_length
}

fn read_u32_bytes(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixture u32"))
}

fn append_zeros(core: &mut Vec<u8>, length: usize) -> u32 {
    let offset = core.len() as u32;
    core.resize(core.len() + length, 0);
    offset
}

fn finish_binary(core: Vec<u8>, raw: Vec<u8>) -> Vec<u8> {
    let mut bytes = vec![0_u8; FILE_HEADER_SIZE];
    write_u32(&mut bytes, 0, 0);
    write_u32(&mut bytes, 4, core.len() as u32);
    write_u32(&mut bytes, 8, raw.len() as u32);
    bytes.extend_from_slice(&core);
    bytes.extend_from_slice(&raw);
    bytes
}

fn write_model_core(core: &mut [u8], name: &str, root: u32, node_count: u32) {
    write_c_string(core, 0x08, 64, name);
    write_u32_core(core, 0x48, root);
    write_u32_core(core, 0x4c, node_count);
    core[0x72] = 4;
    core[0x73] = 1;
    write_f32_core(core, 0x88, -1.0);
    write_f32_core(core, 0x8c, -2.0);
    write_f32_core(core, 0x90, -3.0);
    write_f32_core(core, 0x94, 1.0);
    write_f32_core(core, 0x98, 2.0);
    write_f32_core(core, 0x9c, 3.0);
    write_f32_core(core, 0xa0, 3.5);
    write_f32_core(core, 0xa4, 1.25);
    write_c_string(core, 0xa8, 64, "null");
}

fn write_node_core(core: &mut [u8], offset: u32, number: u32, name: &str, flags: u32) {
    let offset = offset as usize;
    write_u32_core(core, offset + 0x18, 1);
    write_u32_core(core, offset + 0x1c, number);
    write_c_string(core, offset + 0x20, 32, name);
    write_u32_core(core, offset + 0x6c, flags);
}

fn write_mesh_raw_headers(core: &mut [u8], root: u32) {
    let root = root as usize;
    write_c_string(core, root + 0xe8, 64, "m2a_diffuse");
    write_i32_core(core, root + 0x21c, -1);
    write_i32_core(core, root + 0x228, 0);
    write_i32_core(core, root + 0x22c, 0);
    write_u16_core(core, root + 0x230, 3);
    write_u16_core(core, root + 0x232, 1);
    write_i32_core(core, root + 0x234, 36);
    for offset in [0x238, 0x23c, 0x240] {
        write_i32_core(core, root + offset, -1);
    }
    write_i32_core(core, root + 0x244, 60);
    for offset in [0x248, 0x24c, 0x250, 0x254, 0x258, 0x25c, 0x260] {
        write_i32_core(core, root + offset, -1);
    }
}

fn build_mesh_raw(with_skin: bool, _capacity: usize) -> Vec<u8> {
    let mut raw = vec![0_u8; if with_skin { 168 } else { 96 }];
    for (index, value) in [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
        .into_iter()
        .enumerate()
    {
        write_f32_bytes(&mut raw, index * 4, value);
    }
    for (index, value) in [0.0_f32, 0.0, 1.0, 0.0, 0.0, 1.0].into_iter().enumerate() {
        write_f32_bytes(&mut raw, 36 + index * 4, value);
    }
    for vertex in 0..3 {
        write_f32_bytes(&mut raw, 60 + vertex * 12 + 8, 1.0);
    }
    if with_skin {
        for vertex in 0..3 {
            write_f32_bytes(&mut raw, 96 + vertex * 16, 1.0);
            for bone in 0..4 {
                write_u16_bytes(&mut raw, 144 + vertex * 8 + bone * 2, bone as u16);
            }
        }
    }
    raw
}

fn write_face_core(core: &mut [u8], face: u32) {
    let face = face as usize;
    write_f32_core(core, face + 8, 1.0);
    write_i32_core(core, face + 0x10, 7);
    for offset in [0x14, 0x16, 0x18] {
        write_u16_core(core, face + offset, u16::MAX);
    }
    write_u16_core(core, face + 0x1a, 0);
    write_u16_core(core, face + 0x1c, 1);
    write_u16_core(core, face + 0x1e, 2);
}

fn write_animation_core(
    core: &mut [u8],
    offset: u32,
    name: &str,
    root: u32,
    event: u32,
    event_name: &str,
    length: f32,
) {
    let offset = offset as usize;
    write_c_string(core, offset + 0x08, 64, name);
    write_u32_core(core, offset + 0x48, root);
    write_u32_core(core, offset + 0x4c, 2);
    write_f32_core(core, offset + 0x70, length);
    write_f32_core(core, offset + 0x74, 0.25);
    write_c_string(core, offset + 0x78, 64, "root");
    write_array_core(core, offset + 0xb8, event, 1, 1);
    write_f32_core(core, event as usize, length / 2.0);
    write_c_string(core, event as usize + 4, 32, event_name);
}

fn write_controller_core(
    core: &mut [u8],
    offset: usize,
    kind: i32,
    rows: i16,
    time_index: i16,
    data_index: i16,
    columns: i8,
) {
    write_i32_core(core, offset, kind);
    write_i16_core(core, offset + 4, rows);
    write_i16_core(core, offset + 6, time_index);
    write_i16_core(core, offset + 8, data_index);
    core[offset + 10] = columns as u8;
}

fn write_array_core(core: &mut [u8], offset: usize, pointer: u32, used: u32, allocated: u32) {
    write_u32_core(core, offset, pointer);
    write_u32_core(core, offset + 4, used);
    write_u32_core(core, offset + 8, allocated);
}

fn write_u32_core(core: &mut [u8], offset: usize, value: u32) {
    core[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_i32_core(core: &mut [u8], offset: usize, value: i32) {
    core[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u16_core(core: &mut [u8], offset: usize, value: u16) {
    core[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_i16_core(core: &mut [u8], offset: usize, value: i16) {
    core[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_f32_core(core: &mut [u8], offset: usize, value: f32) {
    core[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_f32_bytes(bytes: &mut [u8], offset: usize, value: f32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u16_bytes(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
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
