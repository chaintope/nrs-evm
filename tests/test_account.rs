use nrs_evm::core::{U256, Account, Address, Storage};
use nrs_evm::hex_util::FromHex;

#[test]
fn test_account_from_json() {
    let expect = Account{
        address: Address::from_hex("dd198a31e1dc7419aa5958097bffd6bdd1626ff1").unwrap(),
        balance: U256::from(0),
        nonce: 0,
        code: vec!(0x60, 0x40, 0x60, 0x80, 0x52),
        storage: Storage::default(),
    };
    let account:Account = serde_json::from_str(r#"
    {
        "address":"dd198a31e1dc7419aa5958097bffd6bdd1626ff1",
        "balance":"0",
        "nonce":0,
        "code":[96, 64, 96, 128, 82],
        "storage":{}
    }
    "#).unwrap();
    assert_eq!(expect, account);
}

#[test]
fn test_account_to_json() {
    let account = Account{
        address: Address::from_hex("dd198a31e1dc7419aa5958097bffd6bdd1626ff1").unwrap(),
        balance: U256::from(0),
        nonce: 0,
        code: vec!(0x60, 0x40, 0x60, 0x80, 0x52),
        storage: Storage::default(),
    };
    let json = serde_json::to_string(&account).unwrap();
    let expect_base = r#"
    {
        "address":"dd198a31e1dc7419aa5958097bffd6bdd1626ff1",
        "balance":"0",
        "nonce":0,
        "code":[96, 64, 96, 128, 82],
        "storage":{}
    }
    "#;
    let mut expect = String::from(expect_base);
    expect.retain(|c| c != ' ');
    expect.retain(|c| c!= '\n');
    assert_eq!(expect, json);
}
