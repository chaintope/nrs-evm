use nrs_evm::core::Word;
use nrs_evm::hex_util::ToHex;

#[test]
fn test_word_to_json() {
    let word = Word::from(&[
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1a, 0x1b, 0x1c, 0x00, 0x00, 0x00, 0x00,
    ]);

    let json = serde_json::to_string(&word).unwrap();
    assert_eq!(json, "\"0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c00000000\"");
}

#[test]
fn test_word_from_json() {
    let word: Word = serde_json::from_str("\"0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c00000000\"").unwrap();
    assert_eq!(word.to_hex(), "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c00000000");
}
