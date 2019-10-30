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
            self.0.write(&[0])?;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Word {
    raw: [u8; WORD_BYTE_SIZE]
}

impl Word {
    pub const SIZE: usize = WORD_BYTE_SIZE;
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

impl<'de> Deserialize<'de> for Word {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        use serde::de::Error;

        let s: &str = Deserialize::deserialize(deserializer)?;
        let mut raw = [0; WORD_BYTE_SIZE];
        let bytes = hex::decode(s).map_err(Error::custom)?;
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

impl std::fmt::Binary for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:>064b}", &self.0[3])?;
        write!(f, "{:>064b}", &self.0[2])?;
        write!(f, "{:>064b}", &self.0[1])?;
        write!(f, "{:>064b}", &self.0[0])?;
        Ok(())
    }
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

#[test]
fn test_actual_byte_size() {
    let num = U256::from(1_u32);
    assert_eq!(num.actual_byte_size(), 1);

    let num = U256::from(0);
    assert_eq!(num.actual_byte_size(), 1);

    let num = U256::from(257);
    assert_eq!(num.actual_byte_size(), 2);

    let num = U256::from([
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    assert_eq!(num.actual_byte_size(), 32);
}

#[test]
fn test_ngative() {
    let num = U256::from(2).to_negative();
    assert_eq!(num, U256::from([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    ]));

    let num = U256::from([
        0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ]).to_negative();
    assert_eq!(num, U256::from([
        0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    ]));

    // if the num is negative, then no modify.
    let num = U256::from([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    ]).to_negative();
    assert_eq!(num, U256::from([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    ]));
}

#[test]
fn test_abs() {
    let num = U256::from(2).abs();
    assert_eq!(num, U256::from(2));

    let num = U256::from([
        0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ]).abs();
    assert_eq!(num, U256::from([
        0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ]));

    let num = U256::from([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    ]).abs();
    assert_eq!(num, U256::from(2));
}
