use nrs_evm::core::{Account, Address, OnMemoryWorldState, StorageStatus, Word, WorldStateInterface};
use nrs_evm::hex_util::FromHex;

#[test]
fn test_set_storage() {
    let mut wstate = OnMemoryWorldState::default();
    let account = Account::default();
    let address = Address::from_hex("dd198a31e1dc7419aa5958097bffd6bdd1626ff1").unwrap();
    wstate.insert(address.clone(), account);

    let key = Word::from_hex("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    let value = Word::from_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
    assert_eq!(wstate.set_storage(&address, &key, value), StorageStatus::StorageAdded);
    assert_eq!(wstate.set_storage(&address, &key, value), StorageStatus::StorageUnchanged);

    let value2 = Word::from_hex("0000000000000000000000000000000000000000000000000000000000000002").unwrap();
    assert_eq!(wstate.set_storage(&address, &key, value2), StorageStatus::StorageModified);
    assert_eq!(wstate.set_storage(&address, &key, Word::ZERO.clone()), StorageStatus::StorageDeleted);
    assert_eq!(wstate.set_storage(&address, &key, value), StorageStatus::StorageAdded);
}
#[test]
fn test_get_storage() {
    let mut wstate = OnMemoryWorldState::default();
    let mut account = Account::default();
    let key = Word::from_hex("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    let value = Word::from_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
    account.storage.0.insert(key, value);

    let mut account2 = Account::default();
    let value2 = Word::from_hex("0000000000000000000000000000000000000000000000000000000000000008").unwrap();
    account2.storage.0.insert(key, value2);

    let address = Address::from_hex("dd198a31e1dc7419aa5958097bffd6bdd1626ff1").unwrap();
    let address2 = Address::from_hex("afc01a31e1dc7419aa5958097bffd6bdd1626ff1").unwrap();
    wstate.insert(address.clone(), account);
    wstate.insert(address2.clone(), account2);

    assert_eq!(wstate.get_storage(&address, &key), value);
    assert_eq!(wstate.get_storage(&address2, &key), value2);
}
