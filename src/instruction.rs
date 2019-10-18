use crate::Context;

pub trait OpcodeFn {
    fn gasCost(&self) -> u128;
    fn exec(&self, mut ctx: Context) -> Context;
}

pub struct OpPush1;
pub struct OpPush2;
pub struct OpPush3;
pub struct OpPush4;
pub struct OpPush5;
pub struct OpPush6;
pub struct OpPush7;
pub struct OpPush8;
pub struct OpPush9;
pub struct OpInvalid;

fn convert_u128(data: &[u8], size: usize) -> u128 {
    let mut bytes: [u8; 16] = [0; 16];
    let mut offset = 16 - data.len();
    for b in data {
        bytes[offset] = *b;
        offset += 1;
    }
    let res = u128::from_be_bytes(bytes);
    res
}

trait OpPushGeneral {
    fn data_size(&self) -> u8;
    fn push_exec(&self, mut ctx: Context) -> Context {
        let start = ctx.pc+1;
        let end = ctx.pc+1+self.data_size() as usize;
        let data = &ctx.codes[start..end];
        let mut bytes: &mut[u8] = &mut [0;16];

        ctx.stack.push(convert_u128(&ctx.codes[start..end], self.data_size() as usize));
        ctx.pc += 1 + self.data_size() as usize;
        ctx
    }
}

impl<T: OpPushGeneral> OpcodeFn for T {
    fn gasCost(&self) -> u128 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        self.push_exec(ctx)
    }
}

impl OpPushGeneral for OpPush1 { fn data_size(&self) -> u8 {1} }
impl OpPushGeneral for OpPush2 { fn data_size(&self) -> u8 {2} }

impl OpcodeFn for OpInvalid {
    fn gasCost(&self) -> u128 { 0 }

    fn exec(&self, mut ctx: Context) -> Context {
        ctx
    }
}

pub enum OpcodeInstances {
    OP_PUSH1(OpPush1),
    OP_INVALID(OpInvalid),
}

pub fn decode_op(opcode: u8) -> Box<dyn OpcodeFn> {
    match opcode {
        0x60 => Box::new(OpPush1),
        0x61 => Box::new(OpPush2),
        _ => Box::new(OpInvalid),
    }
}