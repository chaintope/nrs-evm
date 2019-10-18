use crate::instruction::{OpcodeInstances, OpcodeFn, decode_op};
use std::io::{Read, Write};

pub mod instruction;

fn main() {
    println!("Hello, world!");
}

pub struct Context {
    codes: Vec<u8>,
    pc: usize,
    stack: Vec<u128>,
    remaining_gas: u128,
    refund_gas: u128,
    used_gas: u128,
}

impl Context {
    fn dump_stack(&self) {
        println!("stack: {:?}", self.stack)
    }
}

fn execute(opecodes: Vec<u8>) {
    let max_pc = opecodes.len();
    println!("{:?}", opecodes);
    let mut ctx = Context {
        codes: opecodes,
        pc: 0,
        stack: Vec::new(),
        remaining_gas: 0,
        refund_gas: 0,
        used_gas: 9,
    };

    print!("before ");
    ctx.dump_stack();
    while ctx.pc < max_pc {
        ctx = decode_op(ctx.codes[ctx.pc]).exec(ctx);
    }
    print!("after ");
    ctx.dump_stack();
}

#[test]
fn test_execute() {
    execute(vec![0x60, 0x03, 0x61, 0x12, 0x34]);
}