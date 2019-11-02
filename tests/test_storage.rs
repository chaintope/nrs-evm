use std::collections::HashMap;

use nrs_evm::core::{Storage, U256, Word};

#[test]
fn test_strage_to_json() {
    let mut storage = Storage(HashMap::new());
    storage.0.insert(Word::from(U256::from(1_u32)), Word::from_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap());
    let json = serde_json::to_string(&storage).unwrap();
    assert_eq!(r#"{"0000000000000000000000000000000000000000000000000000000000000001":"0000000000000000000000000000000000000000000000000000000000000001"}"#, json);
}

#[test]
fn test_strage_from_json() {
    let storage_json = r#"{"0000000000000000000000000000000000000000000000000000000000000001":"0000000000000000000000000000000000000000000000000000000000000001"}"#;
    let storage: Storage = serde_json::from_str(storage_json).unwrap();
    assert_eq!(storage.0.keys().len(), 1_usize);
    assert!(storage.0.contains_key(&Word::from(U256::from(1))));

    let storage_json = r#"{"0000000000000000000000000000000000000000000000000000000000000002":"0000000000000000000000000000000000000000000000000000000000000002","0000000000000000000000000000000000000000000000000000000000000001":"0000000000000000000000000000000000000000000000000000000000000001"}"#;
    let storage: Storage = serde_json::from_str(storage_json).unwrap();
    assert_eq!(storage.0.keys().len(), 2_usize);
    assert!(storage.0.contains_key(&Word::from(U256::from(1))));
    assert!(storage.0.contains_key(&Word::from(U256::from(2))));

    let expect_value1 = Word::from_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
    let expect_value2 = Word::from_hex("0000000000000000000000000000000000000000000000000000000000000002").unwrap();
    assert_eq!(storage.0.get(&Word::from(U256::from(1))), Some(&expect_value1));
    assert_eq!(storage.0.get(&Word::from(U256::from(2))), Some(&expect_value2));
}
