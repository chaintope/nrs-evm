use std::io::{Cursor, SeekFrom, Seek, Write, Read};

#[derive(Default)]
pub struct Memory(Cursor<Vec<u8>>);

pub trait OffsetWrite<T> {
    fn write(&mut self, offset: u64, data: T) -> std::io::Result<usize>;
}

impl Memory {
    pub fn new() -> Self {
        Memory(Cursor::new(Vec::new()))
    }

    pub fn read(&mut self, offset: u64) -> std::io::Result<Word> {
        let start = offset as usize;
        let end = start + Word::SIZE;
        if self.len() < end {
            self.0.seek(SeekFrom::Start((end - 1) as u64))?;
            self.0.write(&[0])?;
        }
        println!("len={}, end={}", self.len(), end);
        let mut buf: [u8; 32] = [0; 32];
        buf.copy_from_slice(&self.0.get_ref()[start..end]);
        Ok(Word { raw: buf })
    }

    pub fn len(&self) -> usize {
        self.0.get_ref().len()
    }

    fn word_size(&self) -> usize {
        (self.len() + Word::SIZE - 1) / Word::SIZE
    }

    pub fn gas_cost(&self) -> u64 {
        let word_size = self.word_size();
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
fn test_memory_read() {
    let mut memory = Memory::new();
    assert_eq!(memory.len(), 0);
    assert_eq!(memory.gas_cost(), 0);

    let word = memory.read(0_u64).unwrap();
    assert_eq!(memory.len(), 32);
    assert_eq!(U256::from(word), U256::from(0));
    assert_eq!(memory.gas_cost(), 3);

    let word = memory.read(2048_u64).unwrap();
    assert_eq!(memory.len(), 2080);
    assert_eq!(U256::from(word), U256::from(0));
    assert_eq!(memory.gas_cost(), 203);

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

construct_uint! {
    pub struct U256(4);
}

const WORD_BYTE_SIZE: usize = 32;
const NEGATIVE_BIT: u64 = std::u64::MAX / 2 + 1;

#[derive(Debug, Clone, Copy)]
pub struct Word {
    raw: [u8; WORD_BYTE_SIZE]
}

impl Word {
    pub const SIZE: usize = WORD_BYTE_SIZE;
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
