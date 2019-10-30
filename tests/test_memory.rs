use nrs_evm::core::{U256, Memory, OffsetWrite, Word};

#[test]
fn test_memory_write_offset() {
    let mut memory = Memory::new();
    assert_eq!(memory.len(), 0);
    assert_eq!(memory.gas_cost(), 0);

    let bytes: &[u8] = &[1; 32];
    memory.write(0_u64, bytes).unwrap();
    assert_eq!(memory.len(), 32);
    assert_eq!(memory.gas_cost(), 3);

    memory.write(1024_u64, bytes).unwrap();
    assert_eq!(memory.len(), 1056);
    assert_eq!(&memory.as_ref()[32..36], &[0_u8, 0, 0, 0]);
    assert_eq!(&memory.as_ref()[1022..1026], &[0_u8, 0, 1, 1]);
    assert_eq!(memory.gas_cost(), 101);

    let _res = memory.write(28_u64, &[10_u8, 11, 12, 13]).unwrap();
    assert_eq!(memory.len(), 1056);
    assert_eq!(&memory.as_ref()[22..32], &[1_u8, 1, 1, 1, 1, 1, 10, 11, 12, 13]);
    assert_eq!(memory.gas_cost(), 101);
}

#[test]
fn test_memory_allocate() {
    let mut memory = Memory::new();
    assert_eq!(memory.len(), 0);
    assert_eq!(memory.gas_cost(), 0);

    memory.allocate(32).unwrap();
    assert_eq!(memory.len(), 32);
    assert_eq!(memory.gas_cost(), 3);

    memory.allocate(2080).unwrap();
    assert_eq!(memory.len(), 2080);
    assert_eq!(memory.gas_cost(), 203);
}

#[test]
fn test_memory_read() {
    let mut memory = Memory::new();
    assert_eq!(memory.len(), 0);
    assert_eq!(memory.gas_cost(), 0);

    memory.allocate(2080).unwrap();
    let word = memory.read(0_u64).unwrap();
    assert_eq!(U256::from(word), U256::from(0));

    let word = memory.read(2048_u64).unwrap();
    assert_eq!(U256::from(word), U256::from(0));

    // write address
    let data_bytes: &[u8; 32] = &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x9E, 0x9C, 0x03, 0x05,
        0xD3, 0x73, 0x0E, 0xa2, 0x5C, 0x9a, 0x8C, 0xcf,
        0xd7, 0xE9, 0x3b, 0x0a, 0x9c, 0x5A, 0xA4, 0x67,
    ];
    let data = Word::from(data_bytes);
    memory.write(32, data).unwrap();
    let word = memory.read(32_u64).unwrap();
    assert_eq!(memory.len(), 2080);
    assert_eq!(word.as_ref(), data_bytes);

    let word = memory.read(44_u64).unwrap();
    let expect: &[u8; 32] = &[
        0x9E, 0x9C, 0x03, 0x05,
        0xD3, 0x73, 0x0E, 0xa2, 0x5C, 0x9a, 0x8C, 0xcf,
        0xd7, 0xE9, 0x3b, 0x0a, 0x9c, 0x5A, 0xA4, 0x67,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];
    assert_eq!(word.as_ref(), expect);
}

#[test]
fn test_memory_read_multi_bytes() {
    let mut memory = Memory::new();
    assert_eq!(memory.len(), 0);
    assert_eq!(memory.gas_cost(), 0);

    memory.allocate(1030).unwrap();
    let data = memory.read_multi_bytes(0_u64, 1).unwrap();
    assert_eq!(data, &[0_u8]);


    let data = memory.read_multi_bytes(1024_u64, 6).unwrap();
    assert_eq!(data, &[0_u8, 0, 0, 0, 0, 0]);
}
