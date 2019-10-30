extern crate hex;
extern crate keccak_hasher;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate uint;

use crate::core::*;
use crate::instruction::decode_op;

pub mod instruction;
pub mod core;
#[macro_use]
pub mod hex_util;

serialize_as_hex_str!(Word Address);
deserialize_from_hex!(Address);

#[derive(Debug)]
pub enum ContextState {
    Processing,
    Success,
    Revert,
    Invalid,
    OutOfGas,
}
impl Default for ContextState {
    fn default() -> Self {
        ContextState::Processing
    }
}

#[derive(Default, Debug)]
pub struct Context {
    state: ContextState,
    codes: Vec<u8>,
    pc: usize,
    stack: Vec<Word>,
    memory: Memory,
    remaining_gas: u64,
    refund_gas: u64,
    used_gas: u64,
}

impl Context {
    pub fn dump_stack(&self) {
        println!("stack: {:?}", self.stack)
    }
}

pub fn execute(opecodes: Vec<u8>, remaining_gas: u64) -> Context {
    let max_pc = opecodes.len();
    println!("{:?}", opecodes);
    let mut ctx = Context {
        codes: opecodes,
        remaining_gas,
        .. Context::default()
    };

    while ctx.pc < max_pc {
        ctx = decode_op(ctx.codes[ctx.pc]).instruct(ctx);
        ctx.dump_stack();
    }
    ctx
}

mod tests {
    use crate::core::U256;
    use crate::execute;
    use crate::hex_util::ToHex;

    const U256_MAX_BYTES: [u8; 32] = [
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];

    #[test]
    fn test_execute() {
        execute(vec![0x60, 0x01,
                     0x7f,
                     0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                     0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                     0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                     0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                     0x03], 100000);
        construct_uint! {
	    pub struct U256(4);
    }
        let bytes: &[u8; 32] = &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let p = U256::from_big_endian(
            bytes
        );
        let buf: &mut [u8] = &mut [0; 32];
        p.to_big_endian(buf);
        println!("u={:?}", (p.overflowing_add(U256::from(2_u64))));
    }

    #[test]
    fn test_overflow_add() {
        let mut ctx = execute(vec![
            0x60, 0x01,
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0x01], 100000);

        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0_u32));
    }

    #[test]
    fn test_overflow_sub() {
        let mut ctx = execute(vec![
            0x60, 0x0a,
            0x60, 0x01,
            0x03], 100000);

        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from([
            0xff_u8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf7, ]));
    }

    #[test]
    fn test_overflow_mul() {
        let mut ctx = execute(vec![
            0x7f,
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x60, 0x02,
            0x02], 100000);

        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0_u32));
    }

    #[test]
    fn test_0_div() {
        let mut ctx = execute(vec![
            0x7f,
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x60, 0x00,
            0x04], 100000);

        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0_u32));
        let mut ctx = execute(vec![
            0x60, 0x00,
            0x60, 0x04,
            0x04], 100000);

        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0_u32));
    }

    #[test]
    fn test_sdiv() {
        // -2 / -2
        let mut ctx = execute(vec![
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x05], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));

        // -2 / 2
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x05], 100000);

        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(U256_MAX_BYTES));

        // 2 / -2
        let mut ctx = execute(vec![
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x60, 0x02,
            0x05], 100000);

        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(U256_MAX_BYTES));

        // 2 / 2
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x60, 0x02,
            0x05], 100000);

        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));
    }

    #[test]
    fn test_mod() {
        // 7 mod 3
        let mut ctx = execute(vec![
            0x60, 0x03,
            0x60, 0x07,
            0x06], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1_u32));

        // 8 mod 3
        let mut ctx = execute(vec![
            0x60, 0x03,
            0x60, 0x08,
            0x06], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(2_u32));

        // 8 mod 9
        let mut ctx = execute(vec![
            0x60, 0x09,
            0x60, 0x08,
            0x06], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(8_u32));

        // 7 mod 0
        let mut ctx = execute(vec![
            0x60, 0x00,
            0x60, 0x07,
            0x06], 100000);

        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0_u32));
    }

    #[test]
    fn test_smod() {
        // 7 mod 3
        let mut ctx = execute(vec![
            0x60, 0x03,
            0x60, 0x07,
            0x07], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1_u32));

        // 7 mod 0
        let mut ctx = execute(vec![
            0x60, 0x00,
            0x60, 0x07,
            0x07], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0_u32));

        // -7 mod 3
        let mut ctx = execute(vec![
            0x60, 0x03,
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf9,
            0x07], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(U256_MAX_BYTES));
    }

    #[test]
    fn test_addmod() {
        // 2 + 5 mod 3
        let mut ctx = execute(vec![
            0x60, 0x03,
            0x60, 0x05,
            0x60, 0x02,
            0x08], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1_u32));

        // 2 + 5 mod 0
        let mut ctx = execute(vec![
            0x60, 0x00,
            0x60, 0x05,
            0x60, 0x02,
            0x08], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0_u32));
    }

    #[test]
    fn test_mulmod() {
        // 2 * 4 mod 3
        let mut ctx = execute(vec![
            0x60, 0x03,
            0x60, 0x04,
            0x60, 0x02,
            0x09], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(2_u32));

        // 2 * 4 mod 0
        let mut ctx = execute(vec![
            0x60, 0x00,
            0x60, 0x04,
            0x60, 0x02,
            0x09], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0_u32));

        //  overflowing 7 mod 3
        let mut ctx = execute(vec![
            0x60, 0x03,
            0x7f,
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
            0x60, 0x02,
            0x09], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(2_u32));
    }

    #[test]
    fn test_exp() {
        // 2 ^ 3
        let mut ctx = execute(vec![
            0x60, 0x03,
            0x60, 0x02,
            0x0a], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(8_u32));
        assert_eq!(ctx.used_gas, 66);

        // 2 ^ 0
        let mut ctx = execute(vec![
            0x60, 0x00,
            0x60, 0x02,
            0x0a], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1_u32));
        // 0^2
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x60, 0x00,
            0x0a], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0_u32));

        // 2^255
        let mut ctx = execute(vec![
            0x60, 0xff,
            0x60, 0x02,
            0x0a], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from([
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]));

        // 3^115792089237316195423570985008687907853269984665640564039457584007913129639935
        let mut ctx = execute(vec![
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0x60, 0x03,
            0x0a], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()),
                   U256::from_dec_str("77194726158210796949047323339125271902179989777093709359638389338608753093291").unwrap());
        assert_eq!(ctx.used_gas, 1616);
    }

    #[test]
    fn test_signextend() {
        // -1_i16 signext 2
        let mut ctx = execute(vec![
            0x61, 0xff, 0xff,
            0x60, 0x01,
            0x0b], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(U256_MAX_BYTES));
        println!("bool to u256={:?}", U256::from((3 < 2) as u8));
    }

    #[test]
    fn test_lt() {
        // 1 < 2
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x60, 0x01,
            0x10], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));

        // 2 < 2
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x60, 0x02,
            0x10], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));

        // 2 < 1
        let mut ctx = execute(vec![
            0x60, 0x01,
            0x60, 0x02,
            0x10], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));
    }

    #[test]
    fn test_gt() {
        // 1 > 2
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x60, 0x01,
            0x11], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));

        // 2 > 2
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x60, 0x02,
            0x11], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));

        // 2 > 1
        let mut ctx = execute(vec![
            0x60, 0x01,
            0x60, 0x02,
            0x11], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));
    }

    #[test]
    fn test_slt() {
        // -2 < 1
        let mut ctx = execute(vec![
            0x60, 0x01,
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x12], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));

        // 2 < 1
        let mut ctx = execute(vec![
            0x60, 0x01,
            0x60, 0x02,
            0x12], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));
        // -2 < -1
        let mut ctx = execute(vec![
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x12], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));

        // -2 < -2
        let mut ctx = execute(vec![
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x12], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));
    }

    #[test]
    fn test_sgt() {
        // -2 > 1
        let mut ctx = execute(vec![
            0x60, 0x01,
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x13], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));

        // 2 > 1
        let mut ctx = execute(vec![
            0x60, 0x01,
            0x60, 0x02,
            0x13], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));

        // -2 > -1
        let mut ctx = execute(vec![
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x13], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));

        // -2 > -2
        let mut ctx = execute(vec![
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x7f,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0x13], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));
    }

    #[test]
    fn test_eq() {
        // 1 == 1
        let mut ctx = execute(vec![
            0x60, 0x01,
            0x60, 0x01,
            0x14], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));

        // 1 == 2
        let mut ctx = execute(vec![
            0x60, 0x01,
            0x60, 0x02,
            0x14], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));
    }

    #[test]
    fn test_iszero() {
        let mut ctx = execute(vec![
            0x60, 0x00,
            0x15], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));

        let mut ctx = execute(vec![
            0x60, 0x01,
            0x15], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));
    }

    #[test]
    fn test_and() {
        let mut ctx = execute(vec![
            0x61, 0xff, 0xff,
            0x60, 0x01,
            0x16], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));
    }

    #[test]
    fn test_or() {
        let mut ctx = execute(vec![
            0x61, 0xff, 0xff,
            0x60, 0x01,
            0x17], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(65535));
    }

    #[test]
    fn test_xor() {
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x60, 0x01,
            0x18], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(3));
    }

    #[test]
    fn test_not() {
        let mut ctx = execute(vec![
            0x60, 0x01,
            0x19], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(U256_MAX_BYTES) - 1);
    }

    #[test]
    fn test_byte() {
        let mut ctx = execute(vec![
            0x61, 0x08, 0x01,
            0x60, 0x1e,
            0x1a], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(8));
    }

    #[test]
    fn test_shl() {
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x60, 0x01,
            0x1b], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(4));
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x60, 0x02,
            0x1b], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(8));
    }

    #[test]
    fn test_shr() {
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x60, 0x01,
            0x1c], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));
        let mut ctx = execute(vec![
            0x60, 0x04,
            0x60, 0x02,
            0x1c], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));
    }

    #[test]
    fn test_sar() {
        let mut ctx = execute(vec![
            0x60, 0x02,
            0x60, 0x01,
            0x1d], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(1));
        let mut ctx = execute(vec![
            0x7f,
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x60, 0xff,
            0x1d], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(U256_MAX_BYTES));
        let mut ctx = execute(vec![
            0x7f,
            0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x60, 0xff,
            0x1d], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(0));
    }

    #[test]
    fn test_mstore() {
        let mut ctx = execute(vec![
            0x60, 0x80,
            0x60, 0x40,
            0x52], 100000);
        assert!(ctx.stack.pop().is_none());
        let expect: &[u8] = &[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
        ];
        assert_eq!(ctx.memory.as_ref(), expect);
        assert_eq!(ctx.used_gas, 18);
    }

    #[test]
    fn test_mload() {
        let mut ctx = execute(vec![
            0x60, 0x80,
            0x60, 0x40,
            0x52,
            0x60, 0x40,
            0x51], 100000);
        assert_eq!(U256::from(ctx.stack.pop().unwrap()), U256::from(128));
        let expect: &[u8] = &[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
        ];
        assert_eq!(ctx.memory.as_ref(), expect);
        assert_eq!(ctx.used_gas, 24);
    }

    #[test]
    fn test_mstore8() {
        let mut ctx = execute(vec![
            0x63, 0x10, 0x20, 0x30, 0x40,
            0x60, 0x40,
            0x53,
        ], 100000);
        assert!(ctx.stack.pop().is_none());
        let expect: &[u8] = &[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

            0x40,
        ];
        assert_eq!(ctx.memory.as_ref(), expect);
        assert_eq!(ctx.used_gas, 18);
    }

    #[test]
    fn test_sha3() {
        let mut ctx = execute(vec![
            0x63,
            0x74, 0x65, 0x73, 0x74, // test
            0x60, 0x40,
            0x52,
            0x60, 0x04,
            0x60, 0x5c,
            0x20,
        ], 100000);
        assert_eq!(ctx.stack.pop().unwrap().to_hex(), "9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658");
        assert_eq!(ctx.used_gas, 60);
    }
}