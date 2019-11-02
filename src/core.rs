use std::collections::HashMap;
use std::io::{Cursor, Seek, SeekFrom, Write};

use hex::FromHexError;
use serde::{Deserializer, Serializer};
use serde::de::Deserialize;
use serde::ser::Serialize;
use crate::hex_util::{ToHex, FromHex};


///////////////////////////////////////////////
//////////   Memory Implementation    /////////
///////////////////////////////////////////////
#[derive(Default, Debug)]
pub struct Memory(Cursor<Vec<u8>>);

pub trait OffsetWrite<T> {
    fn write(&mut self, offset: u64, data: T) -> std::io::Result<usize>;
}

pub fn word_size(size: usize) -> usize {
    (size + Word::SIZE - 1) / Word::SIZE
}

impl Memory {
    pub fn new() -> Self {
        Memory(Cursor::new(Vec::new()))
    }

    pub fn allocate(&mut self, size: usize) -> std::io::Result<()> {
        if self.len() < size {
            self.0.seek(SeekFrom::Start((size - 1) as u64))?;
            self.0.write_all(&[0])?;
        }
        Ok(())
    }
    pub fn read(&mut self, offset: u64) -> std::io::Result<Word> {
        let bytes = self.read_multi_bytes(offset, Word::SIZE)?;
        let buf: &mut [u8; Word::SIZE] = &mut [0; Word::SIZE];
        buf.copy_from_slice(&bytes[0..Word::SIZE]);
        Ok(Word::from(buf))
    }

    pub fn read_multi_bytes(&mut self, offset: u64, length: usize) -> std::io::Result<Vec<u8>> {
        let start = offset as usize;
        let end = start + length;
        Ok(Vec::from(&self.0.get_ref()[start..end]))
    }

    pub fn len(&self) -> usize {
        self.0.get_ref().len()
    }

    pub fn gas_cost(&self) -> u64 {
        let word_size = word_size(self.len());
        (3 * word_size + word_size * word_size / 512) as u64
    }
}

impl AsRef<[u8]> for Memory {
    fn as_ref(&self) -> &[u8] {
        self.0.get_ref()
    }
}

impl<T: AsRef<[u8]>> OffsetWrite<T> for Memory {
    fn write(&mut self, offset: u64, data: T) -> std::io::Result<usize> {
        self.0.seek(SeekFrom::Start(offset))?;
        self.0.write(data.as_ref())
    }
}


///////////////////////////////////////////////
//////////     U256 Implementation    /////////
///////////////////////////////////////////////
construct_uint! {
    pub struct U256(4);
}

impl Serialize for U256 {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for U256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        use serde::de::Error;
        let s: &str = Deserialize::deserialize(deserializer)?;
        match U256::from_dec_str(s) {
            Ok(num) => Ok(num),
            Err(e) => Err(Error::custom(format!("{:?}", e)))
        }
    }
}

impl std::fmt::Binary for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:>064b}", &self.0[3])?;
        write!(f, "{:>064b}", &self.0[2])?;
        write!(f, "{:>064b}", &self.0[1])?;
        write!(f, "{:>064b}", &self.0[0])?;
        Ok(())
    }
}

impl U256 {
    pub fn is_negative(&self) -> bool {
        self.0[3] >= NEGATIVE_BIT
    }

    pub fn to_negative(mut self) -> Self {
        if !self.is_negative() {
            self = !self + 1
        }
        self
    }

    pub fn abs(mut self) -> Self {
        if self.is_negative() {
            self = !self + 1
        }
        self
    }

    pub fn actual_byte_size(&self) -> u8 {
        let buf: &mut [u8] = &mut [0; 32];
        self.to_big_endian(buf);
        let mut res = 32;
        for b in &buf[..31] {
            if *b == 0_u8 {
                res -= 1;
            } else {
                break;
            }
        }
        res
    }
}

///////////////////////////////////////////////
//////////     Word Implementation    /////////
///////////////////////////////////////////////
const WORD_BYTE_SIZE: usize = 32;
const NEGATIVE_BIT: u64 = std::u64::MAX / 2 + 1;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Word {
    raw: [u8; WORD_BYTE_SIZE]
}

impl Word {
    pub const SIZE: usize = WORD_BYTE_SIZE;
    pub const ZERO: Word = Word {
        raw: [0_u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ]
    };
    pub fn from_hex(hex_str: &str) -> Result<Self, FromHexError> {
        if hex_str.len() != 64 {
            return Err(FromHexError::InvalidStringLength)
        }
        let to_bytes = hex::decode(hex_str)?;
        let mut raw = [0; WORD_BYTE_SIZE];
        raw.copy_from_slice(&to_bytes);
        return Ok(Word::from(&raw))
    }
}

impl ToHex for Word {
    fn to_hex(&self) -> String {
        hex::encode(&self.raw)
    }
}

impl FromHex<Word> for Word {
    fn from_hex(hex_str: &str) -> Result<Self, FromHexError>  {
        if hex_str.len() != 64 {
            return Err(FromHexError::InvalidStringLength)
        }
        let mut raw = [0; WORD_BYTE_SIZE];
        let bytes = hex::decode(hex_str)?;
        raw.copy_from_slice(&bytes);
        Ok(Word::from(&raw))
    }
}

impl AsRef<[u8]> for Word {
    fn as_ref(&self) -> &[u8] {
        &self.raw
    }
}

pub fn convert_word(data: &[u8], size: usize) -> Word {
    let mut bytes: [u8; Word::SIZE] = [0; Word::SIZE];
    let mut offset = Word::SIZE - size;
    for b in data {
        bytes[offset] = *b;
        offset += 1;
    }
    Word { raw: bytes }
}

impl From<&[u8; WORD_BYTE_SIZE]> for Word {
    fn from(u: &[u8; WORD_BYTE_SIZE]) -> Self {
        Word { raw: *u }
    }
}

impl From<&mut [u8; WORD_BYTE_SIZE]> for Word {
    fn from(u: &mut [u8; WORD_BYTE_SIZE]) -> Self {
        Word { raw: *u }
    }
}

impl From<U256> for Word {
    fn from(u: U256) -> Self {
        let buf: &mut [u8; WORD_BYTE_SIZE] = &mut [0; WORD_BYTE_SIZE];
        u.to_big_endian(buf);
        Word::from(buf)
    }
}

impl From<Word> for U256 {
    fn from(w: Word) -> Self {
        U256::from_big_endian(&w.raw)
    }
}

///////////////////////////////////////////////
//////////  Storage Implementation    /////////
///////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct Storage(pub HashMap<Word, Word>);


///////////////////////////////////////////////
//////////  Address Implementation    /////////
///////////////////////////////////////////////
const ADDRESS_BYTE_SIZE: usize = 20;
#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Address([u8; ADDRESS_BYTE_SIZE]);
impl Address {
    pub const SIZE: usize = ADDRESS_BYTE_SIZE;
}
impl From<Word> for Address {
    fn from(word: Word) -> Self {
        let mut raw: [u8; 20] = [0; 20];
        raw.copy_from_slice(&word.as_ref()[12..32]);
        Address(raw)
    }
}

impl From<&[u8; ADDRESS_BYTE_SIZE]> for Address {
    fn from(buf: &[u8; ADDRESS_BYTE_SIZE]) -> Self {
        let mut raw: [u8; ADDRESS_BYTE_SIZE] = [0; ADDRESS_BYTE_SIZE];
        raw.copy_from_slice(buf);
        Address(raw)
    }
}

impl ToHex for Address {
    fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl FromHex<Address> for Address {
    fn from_hex(hex_str: &str) -> Result<Self, FromHexError> {
        if hex_str.len() != ADDRESS_BYTE_SIZE * 2 {
            Err(FromHexError::InvalidStringLength)
        } else {
            let mut raw = [0; ADDRESS_BYTE_SIZE];
            let bytes = hex::decode(hex_str)?;
            raw.copy_from_slice(&bytes);
            Ok(Address::from(&raw))
        }
    }
}

///////////////////////////////////////////////
//////////  Account Implementation    /////////
///////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct Account {
    pub address: Address,
    pub balance: U256,
    pub nonce: u64,
    pub code: Vec<u8>,
    pub storage: Storage,
}

///////////////////////////////////////////////
////////// Transaction Implementation /////////
///////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionContext {
    pub gas_price: U256,
    pub origin: Address,
    pub block_coinbase: Address,
    pub block_number: i64,
    pub block_timestamp: i64,
    pub block_difficulty: U256,
}

///////////////////////////////////////////////
////////// CallMessage Implementation /////////
///////////////////////////////////////////////
pub enum CallKind {
    Call,
    DelegateCall,
    StaticCall,
    CallCode,
    Create,
    Create2,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CallMessage<'a> {
    pub depth: u32,
    pub gas: i64,
    pub destination: Address,
    pub sender: Address,
    pub input_data: &'a [u8],
    pub value: U256,
    pub create2_salt: Word,
}

///////////////////////////////////////////////
//////////  WordState Implementation  /////////
///////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OnMemoryWorldState(HashMap<Address, Account>);

#[derive(Debug, PartialEq, Eq)]
pub enum StorageStatus {
    StorageUnchanged,
    StorageModified,
    StorageModifiedAgain,
    StorageAdded,
    StorageDeleted,
}

pub trait WorldStateInterface {
    fn account_exists(&self, address: &Address) -> bool;
    fn set_storage(&mut self, address: &Address, key: &Word, value: Word) -> StorageStatus;
    fn get_storage(&self, address: &Address, key: &Word) -> Word;
    fn get_balance(&self, address: &Address) -> U256;
    fn get_code_size(&self, address: &Address) -> usize;
    fn get_code_hash(&self, address: &Address) -> Word;
    fn copy_code(&self, address: &Address, buf: &mut [u8]) -> usize;
    fn selfdestruct(&mut self, address: &Address, beneficiary: &Address) -> bool;
}

impl OnMemoryWorldState {
    pub fn insert(&mut self, address: Address, account: Account) -> Option<Account>{
        self.0.insert(address, account)
    }
}
impl WorldStateInterface for OnMemoryWorldState {
    fn account_exists(&self, address: &Address) -> bool {
        self.0.get(address).is_some()
    }

    fn set_storage(&mut self, address: &Address, key: &Word, value: Word) -> StorageStatus {
        let res = match self.0.get(address) {
            Some(a) => {
                match a.storage.0.get(key) {
                    Some(&Word::ZERO) | None => StorageStatus::StorageAdded,
                    Some(w) => {
                        if value == Word::ZERO {
                            StorageStatus::StorageDeleted
                        } else if &value == w {
                            StorageStatus::StorageUnchanged
                        } else {
                            StorageStatus::StorageModified
                        }
                    },
                }
            },
            None => panic!("set_storage: Account: {:?} is not exists!", &address),
        };
        let account = self.0.get_mut(address).unwrap();
        account.storage.0.insert(key.clone(), value);
        res
    }

    fn get_storage(&self, address: &Address, key: &Word) -> Word {
        match self.0.get(address) {
            Some(a) => a.storage.0.get(key).unwrap_or(&Word::ZERO).clone(),
            None => panic!("get_storage: Account: {:?} is not exists!", &address)
        }
    }

    fn get_balance(&self, _address: &Address) -> U256 {
        unimplemented!()
    }

    fn get_code_size(&self, _address: &Address) -> usize {
        unimplemented!()
    }

    fn get_code_hash(&self, _address: &Address) -> Word {
        unimplemented!()
    }

    fn copy_code(&self, _address: &Address, _buf: &mut [u8]) -> usize {
        unimplemented!()
    }

    fn selfdestruct(&mut self, _address: &Address, _beneficiary: &Address) -> bool {
        unimplemented!()
    }
}
