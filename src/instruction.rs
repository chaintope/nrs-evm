use crate::{Context, ContextState};
use crate::core::*;
use keccak_hasher::KeccakHasher;
use hash_db::Hasher;

type InstructionResult = Result<Context, Context>;

fn out_of_gas(mut ctx: Context) -> Context {
    ctx.state = ContextState::OutOfGas;
    ctx.pc = ctx.codes.len();
    ctx
}

fn invalid(mut ctx: Context) -> Context {
    ctx.state = ContextState::Invalid;
    ctx.pc = ctx.codes.len();
    ctx
}

pub trait OpcodeFn {
    fn gas_cost(&self) -> u64;
    fn exec(&self, ctx: Context) -> Context;
    fn instruct(&self, ctx: Context) -> Context {
        let mut res = self.exec(ctx);
        res.used_gas += self.gas_cost();
        res
    }
}

pub struct OpAdd;

impl OpcodeFn for OpAdd {
    fn gas_cost(&self) -> u64 { 3 }

    fn exec(&self, mut ctx: Context) -> Context {
        let result = U256::from(ctx.stack.pop().unwrap()).overflowing_add(U256::from(ctx.stack.pop().unwrap()));
        ctx.stack.push(Word::from(result.0));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpMul;

impl OpcodeFn for OpMul {
    fn gas_cost(&self) -> u64 { 5 }

    fn exec(&self, mut ctx: Context) -> Context {
        let result = U256::from(ctx.stack.pop().unwrap()).overflowing_mul(U256::from(ctx.stack.pop().unwrap()));
        ctx.stack.push(Word::from(result.0));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpSub;

impl OpcodeFn for OpSub {
    fn gas_cost(&self) -> u64 { 3 }

    fn exec(&self, mut ctx: Context) -> Context {
        let result = U256::from(ctx.stack.pop().unwrap()).overflowing_sub(U256::from(ctx.stack.pop().unwrap()));
        ctx.stack.push(Word::from(result.0));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpDiv;

impl OpcodeFn for OpDiv {
    fn gas_cost(&self) -> u64 { 5 }

    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        if b.is_zero() {
            ctx.stack.push(Word::from(U256::from(0)))
        } else {
            let result = a / b;
            ctx.stack.push(Word::from(result));
        }
        ctx.pc += 1;
        ctx
    }
}

pub struct OpSDiv;

impl OpcodeFn for OpSDiv {
    fn gas_cost(&self) -> u64 { 5 }

    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        if b.is_zero() {
            ctx.stack.push(Word::from(U256::from(0)))
        } else {
            if a.is_negative() ^ b.is_negative() {
                ctx.stack.push(Word::from((a.abs() / b.abs()).to_negative()));
            } else {
                ctx.stack.push(Word::from(a / b));
            }
        }
        ctx.pc += 1;
        ctx
    }
}

pub struct OpMod;

impl OpcodeFn for OpMod {
    fn gas_cost(&self) -> u64 { 5 }

    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        if b.is_zero() {
            ctx.stack.push(Word::from(U256::from(0)))
        } else {
            ctx.stack.push(Word::from(a % b));
        }
        ctx.pc += 1;
        ctx
    }
}

pub struct OpSMod;

impl OpcodeFn for OpSMod {
    fn gas_cost(&self) -> u64 { 5 }

    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        if b.is_zero() {
            ctx.stack.push(Word::from(U256::from(0)))
        } else {
            if a.is_negative() {
                ctx.stack.push(Word::from((a.abs() % b).to_negative()));
            } else {
                ctx.stack.push(Word::from(a % b));
            }
        }
        ctx.pc += 1;
        ctx
    }
}

pub struct OpAddMod;

impl OpcodeFn for OpAddMod {
    fn gas_cost(&self) -> u64 { 8 }

    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        let c = U256::from(ctx.stack.pop().unwrap());
        if c.is_zero() {
            ctx.stack.push(Word::from(U256::from(0)))
        } else {
            let d = a.overflowing_add(b).0;
            ctx.stack.push(Word::from(d % c));
        }
        ctx.pc += 1;
        ctx
    }
}

pub struct OpMulMod;

impl OpcodeFn for OpMulMod {
    fn gas_cost(&self) -> u64 { 8 }

    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        let c = U256::from(ctx.stack.pop().unwrap());
        if c.is_zero() {
            ctx.stack.push(Word::from(U256::from(0)))
        } else {
            let d = a.overflowing_mul(b).0;
            ctx.stack.push(Word::from(d % c));
        }
        ctx.pc += 1;
        ctx
    }
}

pub struct OpExp;

impl OpcodeFn for OpExp {
    fn gas_cost(&self) -> u64 { 10 }

    fn exec(&self, mut ctx: Context) -> Context {
        let base = U256::from(ctx.stack.pop().unwrap());
        let exponent = U256::from(ctx.stack.pop().unwrap());
        ctx.stack.push(Word::from(base.overflowing_pow(exponent).0));
        ctx.pc += 1;

        // additional gas cost
        ctx.used_gas += 50 * exponent.actual_byte_size() as u64;
        ctx
    }
}

pub struct OpSignExtend;

impl OpcodeFn for OpSignExtend {
    fn gas_cost(&self) -> u64 { 5 }

    fn exec(&self, mut ctx: Context) -> Context {
        let ext = U256::from(ctx.stack.pop().unwrap());
        if ext < U256::from(31) {
            let base = U256::from(ctx.stack.pop().unwrap());
            let bit = ext * 8 + 7;
            let sign_mask = U256::from(1) << bit;
            let value_mask = sign_mask - 1;
            let is_neg = !(base & sign_mask).is_zero();
            if is_neg {
                ctx.stack.push(Word::from(base | !value_mask));
            } else {
                ctx.stack.push(Word::from(base & value_mask));
            }
        }
        ctx.pc += 1;
        ctx
    }
}

pub struct OpLt;

impl OpcodeFn for OpLt {
    fn gas_cost(&self) -> u64 { 3 }

    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        ctx.stack.push(Word::from(U256::from((a < b) as u8)));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpGt;

impl OpcodeFn for OpGt {
    fn gas_cost(&self) -> u64 { 3 }

    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        ctx.stack.push(Word::from(U256::from((a > b) as u8)));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpSLt;

impl OpcodeFn for OpSLt {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        let neg_a = a.is_negative();
        let neg_b = b.is_negative();
        if neg_a ^ neg_b {
            ctx.stack.push(Word::from(U256::from(neg_a as u8)));
        } else {
            ctx.stack.push(Word::from(U256::from((a < b) as u8)));
        }
        ctx.pc += 1;
        ctx
    }
}

pub struct OpSGt;

impl OpcodeFn for OpSGt {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        let neg_a = a.is_negative();
        let neg_b = b.is_negative();
        if neg_a ^ neg_b {
            ctx.stack.push(Word::from(U256::from(neg_b as u8)));
        } else {
            ctx.stack.push(Word::from(U256::from((a > b) as u8)));
        }
        ctx.pc += 1;
        ctx
    }
}

pub struct OpEq;

impl OpcodeFn for OpEq {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        ctx.stack.push(Word::from(U256::from((a == b) as u8)));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpIsZero;

impl OpcodeFn for OpIsZero {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        ctx.stack.push(Word::from(U256::from((a.is_zero()) as u8)));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpAnd;

impl OpcodeFn for OpAnd {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        ctx.stack.push(Word::from(a & b));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpOr;

impl OpcodeFn for OpOr {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        ctx.stack.push(Word::from(a | b));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpXOr;

impl OpcodeFn for OpXOr {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        let b = U256::from(ctx.stack.pop().unwrap());
        ctx.stack.push(Word::from(a ^ b));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpNot;

impl OpcodeFn for OpNot {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let a = U256::from(ctx.stack.pop().unwrap());
        ctx.stack.push(Word::from(!a));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpByte;

impl OpcodeFn for OpByte {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let n = U256::from(ctx.stack.pop().unwrap());
        let x = U256::from(ctx.stack.pop().unwrap());
        if n > U256::from(31_u8) {
            ctx.stack.push(Word::from(U256::from(0_u8)));
        } else {
            let sh: u32 = (31 - n.low_u32()) * 8;
            let mut y = x >> sh;
            y = y & U256::from(0xff_u8);
            ctx.stack.push(Word::from(y));
        }
        ctx.pc += 1;
        ctx
    }
}

pub struct OpSHL;

impl OpcodeFn for OpSHL {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let sh = U256::from(ctx.stack.pop().unwrap());
        let x = U256::from(ctx.stack.pop().unwrap());
        ctx.stack.push(Word::from(x << sh));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpSHR;

impl OpcodeFn for OpSHR {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let sh = U256::from(ctx.stack.pop().unwrap());
        let x = U256::from(ctx.stack.pop().unwrap());
        ctx.stack.push(Word::from(x >> sh));
        ctx.pc += 1;
        ctx
    }
}

pub struct OpSAR;

impl OpcodeFn for OpSAR {
    fn gas_cost(&self) -> u64 { 3 }
    fn exec(&self, mut ctx: Context) -> Context {
        let sh = U256::from(ctx.stack.pop().unwrap());
        let x = U256::from(ctx.stack.pop().unwrap());
        let value_neg = x.is_negative();
        let u256_256 = U256::from(256);
        if !value_neg { // value is positive. it is same as right shift.
            ctx.stack.push(Word::from(x >> sh));
        } else { // keep top bit.
            let allones = !U256::from(0);
            if sh > u256_256 {
                // cycled. so, all bit is 1.
                ctx.stack.push(Word::from(allones));
            } else {
                let y = (x >> sh) | (allones << (u256_256 - sh));
                ctx.stack.push(Word::from(y));
            }
        }
        ctx.pc += 1;
        ctx
    }
}

pub struct OpSHA3;

impl OpcodeFn for OpSHA3 {
    fn gas_cost(&self) -> u64 { 30 }

    fn exec(&self, mut ctx: Context) -> Context {
        let index = U256::from(ctx.stack.pop().unwrap());
        let size = U256::from(ctx.stack.pop().unwrap());
        match memory_allocation_check_u256(ctx, index, size) {
            Ok(mut ctx) => {
                let input = ctx.memory.read_multi_bytes(index.low_u64(), size.as_usize()).unwrap();
                let hash = KeccakHasher::hash(&input);
                ctx.stack.push(Word::from(&hash));

                // additional gas cost
                ctx.used_gas += (word_size(size.as_usize()) * 6) as u64;
                ctx.pc += 1;
                ctx
            },
            Err(ctx) => ctx
        }
    }
}

// ###############################################################
// #############          Memory Operations          #############
// ###############################################################

fn memory_allocation_check_u256(ctx: Context, offset: U256, size: U256) -> InstructionResult {
    if size > U256::from(std::u32::MAX) {
        Err(out_of_gas(ctx))
    } else {
        memory_allocation_check(ctx, offset, size.as_usize())
    }
}

fn memory_allocation_check(mut ctx: Context, offset: U256, size: usize) -> InstructionResult {
    if offset > U256::from(std::u32::MAX) {
        Err(out_of_gas(ctx))
    } else {
        let current_cost = ctx.memory.gas_cost();
        match ctx.memory.allocate((offset + size).as_usize()) {
            Ok(_) => {
                let new_cost = ctx.memory.gas_cost();
                // additional gas cost
                ctx.used_gas += new_cost - current_cost;
                if ctx.used_gas > ctx.remaining_gas {
                    Err(out_of_gas(ctx))
                } else {
                    Ok(ctx)
                }
            },
            Err(_) => {
                Err(invalid(ctx))
            }
        }
    }
}

pub trait OpMemoryBase {
    fn op_mem_exec(&self, mut ctx: Context) -> Context {
        let offset = U256::from(ctx.stack.pop().unwrap());
        let res = memory_allocation_check(ctx, offset, self.data_size());
        match res {
            Ok(mut ctx) => {
                ctx = self.individual(offset.low_u64(), ctx);
                ctx.pc += 1;
                ctx
            },
            Err(ctx) => ctx,
        }
    }
    fn data_size(&self) -> usize;
    fn individual(&self, offset: u64, ctx: Context) -> Context;
}

pub struct OpMemoryFn<T: OpMemoryBase>(T);

impl<T: OpMemoryBase> OpcodeFn for OpMemoryFn<T> {
    fn gas_cost(&self) -> u64 { 3 }

    fn exec(&self, ctx: Context) -> Context {
        self.0.op_mem_exec(ctx)
    }
}

pub struct OpMLoad;

impl OpMemoryBase for OpMLoad {
    fn data_size(&self) -> usize { Word::SIZE }
    fn individual(&self, offset: u64, mut ctx: Context) -> Context {
        ctx.stack.push(ctx.memory.read(offset).unwrap());
        ctx
    }
}

pub struct OpMStore;

impl OpMemoryBase for OpMStore {
    fn data_size(&self) -> usize { Word::SIZE }
    fn individual(&self, offset: u64, mut ctx: Context) -> Context {
        ctx.memory.write(offset, ctx.stack.pop().unwrap()).unwrap();
        ctx
    }
}

pub struct OpMStore8;

impl OpMemoryBase for OpMStore8 {
    fn data_size(&self) -> usize { 1 }
    fn individual(&self, offset: u64, mut ctx: Context) -> Context {
        let data = ctx.stack.pop().unwrap();
        let onebyte: u8 = data.as_ref()[31];
        ctx.memory.write(offset, &[onebyte]).unwrap();
        ctx
    }
}

pub struct OpInvalid;

impl OpcodeFn for OpInvalid {
    fn gas_cost(&self) -> u64 { 0 }

    fn exec(&self, mut ctx: Context) -> Context {
        ctx.pc = ctx.codes.len();
        ctx
    }
}

// ###############################################################
// #############               OP_PUSH               #############
// ###############################################################

pub struct OpPush1;

pub struct OpPush2;

pub struct OpPush3;

pub struct OpPush4;

pub struct OpPush5;

pub struct OpPush6;

pub struct OpPush7;

pub struct OpPush8;

pub struct OpPush9;

pub struct OpPush10;

pub struct OpPush11;

pub struct OpPush12;

pub struct OpPush13;

pub struct OpPush14;

pub struct OpPush15;

pub struct OpPush16;

pub struct OpPush17;

pub struct OpPush18;

pub struct OpPush19;

pub struct OpPush20;

pub struct OpPush21;

pub struct OpPush22;

pub struct OpPush23;

pub struct OpPush24;

pub struct OpPush25;

pub struct OpPush26;

pub struct OpPush27;

pub struct OpPush28;

pub struct OpPush29;

pub struct OpPush30;

pub struct OpPush31;

pub struct OpPush32;

pub trait OpPushGeneral {
    fn data_size(&self) -> u8;
    fn push_exec(&self, mut ctx: Context) -> Context {
        let start = ctx.pc + 1;
        let end = ctx.pc + 1 + self.data_size() as usize;
        ctx.stack.push(convert_word(&ctx.codes[start..end], self.data_size() as usize));
        ctx.pc += 1 + self.data_size() as usize;
        ctx
    }
}

struct OpPushFn<T: OpPushGeneral>(T);
impl<T: OpPushGeneral> OpcodeFn for OpPushFn<T> {
    fn gas_cost(&self) -> u64 { 3 }

    fn exec(&self, ctx: Context) -> Context {
        self.0.push_exec(ctx)
    }
}

impl OpPushGeneral for OpPush1 { fn data_size(&self) -> u8 { 1 } }

impl OpPushGeneral for OpPush2 { fn data_size(&self) -> u8 { 2 } }

impl OpPushGeneral for OpPush3 { fn data_size(&self) -> u8 { 3 } }

impl OpPushGeneral for OpPush4 { fn data_size(&self) -> u8 { 4 } }

impl OpPushGeneral for OpPush5 { fn data_size(&self) -> u8 { 5 } }

impl OpPushGeneral for OpPush6 { fn data_size(&self) -> u8 { 6 } }

impl OpPushGeneral for OpPush7 { fn data_size(&self) -> u8 { 7 } }

impl OpPushGeneral for OpPush8 { fn data_size(&self) -> u8 { 8 } }

impl OpPushGeneral for OpPush9 { fn data_size(&self) -> u8 { 9 } }

impl OpPushGeneral for OpPush10 { fn data_size(&self) -> u8 { 10 } }

impl OpPushGeneral for OpPush11 { fn data_size(&self) -> u8 { 11 } }

impl OpPushGeneral for OpPush12 { fn data_size(&self) -> u8 { 12 } }

impl OpPushGeneral for OpPush13 { fn data_size(&self) -> u8 { 13 } }

impl OpPushGeneral for OpPush14 { fn data_size(&self) -> u8 { 14 } }

impl OpPushGeneral for OpPush15 { fn data_size(&self) -> u8 { 15 } }

impl OpPushGeneral for OpPush16 { fn data_size(&self) -> u8 { 16 } }

impl OpPushGeneral for OpPush17 { fn data_size(&self) -> u8 { 17 } }

impl OpPushGeneral for OpPush18 { fn data_size(&self) -> u8 { 18 } }

impl OpPushGeneral for OpPush19 { fn data_size(&self) -> u8 { 19 } }

impl OpPushGeneral for OpPush20 { fn data_size(&self) -> u8 { 20 } }

impl OpPushGeneral for OpPush21 { fn data_size(&self) -> u8 { 21 } }

impl OpPushGeneral for OpPush22 { fn data_size(&self) -> u8 { 22 } }

impl OpPushGeneral for OpPush23 { fn data_size(&self) -> u8 { 23 } }

impl OpPushGeneral for OpPush24 { fn data_size(&self) -> u8 { 24 } }

impl OpPushGeneral for OpPush25 { fn data_size(&self) -> u8 { 25 } }

impl OpPushGeneral for OpPush26 { fn data_size(&self) -> u8 { 26 } }

impl OpPushGeneral for OpPush27 { fn data_size(&self) -> u8 { 27 } }

impl OpPushGeneral for OpPush28 { fn data_size(&self) -> u8 { 28 } }

impl OpPushGeneral for OpPush29 { fn data_size(&self) -> u8 { 29 } }

impl OpPushGeneral for OpPush30 { fn data_size(&self) -> u8 { 30 } }

impl OpPushGeneral for OpPush31 { fn data_size(&self) -> u8 { 31 } }

impl OpPushGeneral for OpPush32 { fn data_size(&self) -> u8 { 32 } }

pub fn decode_op(opcode: u8) -> Box<dyn OpcodeFn> {
    match opcode {
        // Arithmetic
        0x01 => Box::new(OpAdd),
        0x02 => Box::new(OpMul),
        0x03 => Box::new(OpSub),
        0x04 => Box::new(OpDiv),
        0x05 => Box::new(OpSDiv),
        0x06 => Box::new(OpMod),
        0x07 => Box::new(OpSMod),
        0x08 => Box::new(OpAddMod),
        0x09 => Box::new(OpMulMod),
        0x0a => Box::new(OpExp),

        // Bit extend
        0x0b => Box::new(OpSignExtend),

        // Compares
        0x10 => Box::new(OpLt),
        0x11 => Box::new(OpGt),
        0x12 => Box::new(OpSLt),
        0x13 => Box::new(OpSGt),
        0x14 => Box::new(OpEq),
        0x15 => Box::new(OpIsZero),

        // Bitwise Operations
        0x16 => Box::new(OpAnd),
        0x17 => Box::new(OpOr),
        0x18 => Box::new(OpXOr),
        0x19 => Box::new(OpNot),
        0x1a => Box::new(OpByte),
        0x1b => Box::new(OpSHL),
        0x1c => Box::new(OpSHR),
        0x1d => Box::new(OpSAR),

        0x20 => Box::new(OpSHA3),
        // Memory Operations
        0x51 => Box::new(OpMemoryFn(OpMLoad)),
        0x52 => Box::new(OpMemoryFn(OpMStore)),
        0x53 => Box::new(OpMemoryFn(OpMStore8)),

        // PUSHx
        0x60 => Box::new(OpPushFn(OpPush1)),
        0x61 => Box::new(OpPushFn(OpPush2)),
        0x62 => Box::new(OpPushFn(OpPush3)),
        0x63 => Box::new(OpPushFn(OpPush4)),
        0x64 => Box::new(OpPushFn(OpPush5)),
        0x65 => Box::new(OpPushFn(OpPush6)),
        0x66 => Box::new(OpPushFn(OpPush7)),
        0x67 => Box::new(OpPushFn(OpPush8)),
        0x68 => Box::new(OpPushFn(OpPush9)),
        0x69 => Box::new(OpPushFn(OpPush10)),
        0x6a => Box::new(OpPushFn(OpPush11)),
        0x6b => Box::new(OpPushFn(OpPush12)),
        0x6c => Box::new(OpPushFn(OpPush13)),
        0x6d => Box::new(OpPushFn(OpPush14)),
        0x6e => Box::new(OpPushFn(OpPush15)),
        0x6f => Box::new(OpPushFn(OpPush16)),
        0x70 => Box::new(OpPushFn(OpPush17)),
        0x71 => Box::new(OpPushFn(OpPush18)),
        0x72 => Box::new(OpPushFn(OpPush19)),
        0x73 => Box::new(OpPushFn(OpPush20)),
        0x74 => Box::new(OpPushFn(OpPush21)),
        0x75 => Box::new(OpPushFn(OpPush22)),
        0x76 => Box::new(OpPushFn(OpPush23)),
        0x77 => Box::new(OpPushFn(OpPush24)),
        0x78 => Box::new(OpPushFn(OpPush25)),
        0x79 => Box::new(OpPushFn(OpPush26)),
        0x7a => Box::new(OpPushFn(OpPush27)),
        0x7b => Box::new(OpPushFn(OpPush28)),
        0x7c => Box::new(OpPushFn(OpPush29)),
        0x7d => Box::new(OpPushFn(OpPush30)),
        0x7e => Box::new(OpPushFn(OpPush31)),
        0x7f => Box::new(OpPushFn(OpPush32)),
        _ => Box::new(OpInvalid),
    }
}