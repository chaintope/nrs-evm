use nrs_evm::core::{Address, Word};
use nrs_evm::hex_util::FromHex;

#[test]
fn test_address_to_json() {
    let word = Word::from_hex("000000000000000000000000dD198A31E1Dc7419AA5958097BFfD6BDd1626FF1").unwrap();
    let address = Address::from(word);
    let json = serde_json::to_string(&address).unwrap();
    assert_eq!(r#""dd198a31e1dc7419aa5958097bffd6bdd1626ff1""#, json);
}
#[test]
fn test_address_from_hex() {
    let word = Word::from_hex("000000000000000000000000dD198A31E1Dc7419AA5958097BFfD6BDd1626FF1").unwrap();
    let address = Address::from(word);
    let addr:Address = Address::from_hex("dd198a31e1dc7419aa5958097bffd6bdd1626ff1").unwrap();
    assert_eq!(address, addr);
}

#[test]
fn test_address_from_json() {
    let word = Word::from_hex("000000000000000000000000dD198A31E1Dc7419AA5958097BFfD6BDd1626FF1").unwrap();
    let address = Address::from(word);
    let addr:Address = serde_json::from_str(r#""dd198a31e1dc7419aa5958097bffd6bdd1626ff1""#).unwrap();
    assert_eq!(address, addr);
}
