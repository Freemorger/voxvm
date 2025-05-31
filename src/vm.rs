#![allow(non_snake_case)]

use crate::fileformats::VoxExeHeader;
use core::panic;
use std::{collections::HashMap, fmt::Result, fs, thread::panicking};

#[derive(Debug, Clone, Copy)]
pub enum RegTypes {
    uint64,
    int64,
    float64,
    StrAddr,
    address,
}

#[derive(Debug)]
pub struct VM {
    registers: [u64; 32],
    reg_types: [RegTypes; 32],
    flags: [u8; 4], // of, zf, nf, cf
    ip: usize,
    memory: Vec<u8>, // dividing by each bytes, then can be grouped
    stack: Vec<u64>,
    sp: u64,       // stack pointer
    heap: Vec<u8>, // same w normal mem
    heap_ptr: u64,
    data_base: u64, // pointer at data segment start
    data_size: u64,
    nativecalls: std::collections::HashMap<u16, NativeFn>,
    running: bool,
    float_epsilon: f64,
}
type NativeFn = fn(&mut VM, &[u64]) -> Result;
type InstructionHandler = fn(&mut VM);

impl VM {
    pub fn new(init_mem: usize, init_stack: usize, init_heap: usize) -> VM {
        VM {
            registers: [0; 32],
            reg_types: [RegTypes::uint64; 32],
            flags: [0; 4],
            ip: 0x0,
            memory: vec![0; init_mem],
            stack: vec![0; init_stack],
            sp: 0x0,
            heap: vec![0; init_heap],
            heap_ptr: 0x0,
            data_base: 0x0,
            data_size: 0,
            nativecalls: HashMap::new(),
            running: true,
            float_epsilon: 1e-10,
        }
    }
    pub fn load_vvr(&mut self, input_file_name: &str) {
        // vvr = voxvm raw
        let mut bctr: usize = 0;
        match fs::read(input_file_name) {
            Ok(bytes) => {
                for byte in &bytes {
                    self.memory[bctr] = *byte;
                    bctr += 1;
                }
            }
            Err(err) => {
                panic!("CRITICAL: Can't read .vvr file. Error: {}", err)
            }
        }
    }

    pub fn load_vve(&mut self, input_file_name: &str, minVveVersion: u16) {
        // vve = voxvm executable
        let fileHeader: VoxExeHeader = VoxExeHeader::load(input_file_name, minVveVersion).unwrap();
        //println!("DBG Entry: {}", fileHeader.entry_point);

        let header_size: usize = 38;
        self.ip = fileHeader.entry_point as usize;
        self.data_base = fileHeader.data_base;
        self.data_size = fileHeader.data_size;
        let mut bctr: usize = 0;

        match fs::read(input_file_name) {
            Ok(bytes) => {
                for byte in &bytes[header_size..] {
                    self.memory[bctr] = *byte;
                    bctr += 1;
                }
            }
            Err(err) => {
                panic!("CRITICAL: Can't read .vve file. Error: {}", err)
            }
        }
    }

    pub fn run(&mut self) {
        while (self.ip < self.memory.capacity()) && (self.running) {
            let opcode = self.memory[self.ip];
            //println!("DBG: cur opcode: {}", self.ip);
            Self::OPERATIONS[opcode as usize](self);
        }
        if (self.ip >= self.memory.capacity()) {
            panic!(
                "CRITICAL: Instruction overflow! VM Memory capacity: {}, latest opcode: {}.
                \n Consider running VM with more init ram using
                --init-ram=RAM_VALUE",
                self.memory.capacity(),
                self.ip
            );
        }
    }

    const OPERATIONS: [InstructionHandler; 256] = {
        let mut handlers = [Self::op_unimplemented as InstructionHandler; 256];
        handlers[0xFF] = Self::op_halt as InstructionHandler;
        handlers[0x01] = Self::op_ncall as InstructionHandler;
        handlers[0x02] = Self::op_nop as InstructionHandler;
        handlers[0x10] = Self::op_uload as InstructionHandler;
        handlers[0x11] = Self::op_uadd as InstructionHandler;
        handlers[0x12] = Self::op_umul as InstructionHandler;
        handlers[0x13] = Self::op_usub as InstructionHandler;
        handlers[0x14] = Self::op_udiv as InstructionHandler;
        handlers[0x15] = Self::op_urem as InstructionHandler;
        handlers[0x16] = Self::op_ucmp as InstructionHandler;
        handlers[0x17] = Self::op_usqrt as InstructionHandler;
        handlers[0x18] = Self::op_upow as InstructionHandler;
        handlers[0x20] = Self::op_iload as InstructionHandler;
        handlers[0x21] = Self::op_iadd as InstructionHandler;
        handlers[0x22] = Self::op_imul as InstructionHandler;
        handlers[0x23] = Self::op_isub as InstructionHandler;
        handlers[0x24] = Self::op_idiv as InstructionHandler;
        handlers[0x25] = Self::op_irem as InstructionHandler;
        handlers[0x26] = Self::op_icmp as InstructionHandler;
        handlers[0x27] = Self::op_iabs as InstructionHandler;
        handlers[0x28] = Self::op_ineg as InstructionHandler;
        handlers[0x29] = Self::op_isqrt as InstructionHandler;
        handlers[0x2a] = Self::op_ipow as InstructionHandler;
        handlers[0x30] = Self::op_fload as InstructionHandler;
        handlers[0x31] = Self::op_fadd as InstructionHandler;
        handlers[0x32] = Self::op_fmul as InstructionHandler;
        handlers[0x33] = Self::op_fsub as InstructionHandler;
        handlers[0x34] = Self::op_fdiv as InstructionHandler;
        handlers[0x35] = Self::op_frem as InstructionHandler;
        handlers[0x36] = Self::op_fcmp as InstructionHandler;
        handlers[0x37] = Self::op_fcmp_eps as InstructionHandler;
        handlers[0x38] = Self::op_fabs as InstructionHandler;
        handlers[0x39] = Self::op_fneg as InstructionHandler;
        handlers[0x3a] = Self::op_fsqrt as InstructionHandler;
        handlers[0x3b] = Self::op_fpow as InstructionHandler;
        handlers[0x40] = Self::op_jmp as InstructionHandler;
        handlers[0x41] = Self::op_jz as InstructionHandler;
        handlers[0x42] = Self::op_jl as InstructionHandler;
        handlers[0x43] = Self::op_jg as InstructionHandler;
        handlers[0x44] = Self::op_jge as InstructionHandler;
        handlers[0x45] = Self::op_jle as InstructionHandler;
        handlers[0x50] = Self::op_utoi as InstructionHandler;
        handlers[0x51] = Self::op_itou as InstructionHandler;
        handlers[0x52] = Self::op_utof as InstructionHandler;
        handlers[0x53] = Self::op_itof as InstructionHandler;
        handlers[0x54] = Self::op_ftou as InstructionHandler;
        handlers[0x55] = Self::op_ftoi as InstructionHandler;
        handlers[0x60] = Self::op_movr as InstructionHandler;
        handlers[0x61] = Self::op_or as InstructionHandler;
        handlers[0x62] = Self::op_and as InstructionHandler;
        handlers[0x63] = Self::op_not as InstructionHandler;
        handlers[0x64] = Self::op_xor as InstructionHandler;
        handlers[0x65] = Self::op_test as InstructionHandler;
        handlers[0x66] = Self::op_lnot as InstructionHandler;
        handlers[0x70] = Self::op_dsload as InstructionHandler;
        // ...
        handlers
    };

    fn op_unimplemented(&mut self) {
        panic!(
            "CRITICAL: Unknown operation code at {}: {}",
            self.ip, self.memory[self.ip]
        );
    }

    fn op_halt(&mut self) {
        self.running = false;
    }

    fn op_ncall(&mut self) {
        // 0x1, size: different
        let ncall_num: u16 = args_to_u16(&self.memory[(self.ip + 1)..(self.ip + 3)]);
        match ncall_num {
            0x1 => self.ncall_println(),
            _ => panic!("Unknown ncall code: {}", ncall_num),
        }
    }

    fn op_nop(&mut self) {
        // 0x2, size: 1
        return;
    }

    fn op_uload(&mut self) {
        // 0x10, size: 10
        let register_ind: u8 = self.memory[(self.ip + 1) as usize];
        let value: u64 = args_to_u64(&self.memory[(self.ip + 2)..(self.ip + 10)]);

        self.registers[register_ind as usize] = value;
        self.reg_types[register_ind as usize] = RegTypes::uint64;
        self.ip += 10;
        return;
    }

    fn op_uadd(&mut self) {
        // 0x11, size: 3
        let in_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
        let toadd_reg_ind: u8 = self.memory[(self.ip + 2) as usize];

        self.registers[in_reg_ind as usize] += self.registers[toadd_reg_ind as usize];
        self.ip += 3;
        return;
    }

    fn op_umul(&mut self) {
        // 0x12, size: 3
        let in_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
        let toadd_reg_ind: u8 = self.memory[(self.ip + 2) as usize];

        self.registers[in_reg_ind as usize] *= self.registers[toadd_reg_ind as usize];
        self.ip += 3;
        return;
    }

    fn op_usub(&mut self) {
        // 0x13, size: 3
        let in_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
        let toadd_reg_ind: u8 = self.memory[(self.ip + 2) as usize];

        self.registers[in_reg_ind as usize] -= self.registers[toadd_reg_ind as usize];
        if (self.registers[in_reg_ind as usize] == 0) {
            self.flags[1] = 1;
        } else {
            self.flags[1] = 0;
        }
        self.ip += 3;
        return;
    }

    fn op_udiv(&mut self) {
        // 0x14, size: 4
        let reg_out: u8 = self.memory[(self.ip + 1)];
        let reg_1: u8 = self.memory[(self.ip + 2)];
        let reg_2: u8 = self.memory[(self.ip + 3)];
        if (self.registers[reg_2 as usize] == 0) {
            panic!("DIVZERO Exception at addr {}", self.ip);
        }

        self.registers[reg_out as usize] =
            self.registers[reg_1 as usize] / self.registers[reg_2 as usize];

        self.reg_types[reg_out as usize] = RegTypes::uint64;

        self.ip += 4;
    }

    fn op_urem(&mut self) {
        // 0x15, size: 4
        let reg_dest: u8 = self.memory[(self.ip + 1)];
        let reg_1: u8 = self.memory[(self.ip + 2)];
        let reg_2: u8 = self.memory[(self.ip + 3)];

        self.registers[reg_dest as usize] =
            self.registers[reg_1 as usize] % self.registers[reg_2 as usize];

        self.reg_types[reg_dest as usize] = RegTypes::uint64;

        self.ip += 4;
    }

    fn op_ucmp(&mut self) {
        // 0x16, size: 3
        let reg_dest: u8 = self.memory[(self.ip + 1)];
        let reg_src: u8 = self.memory[(self.ip + 2)];

        let isLess: bool = (self.registers[reg_dest as usize] < self.registers[reg_src as usize]);
        let isEqu: bool = (self.registers[reg_dest as usize] == self.registers[reg_src as usize]);

        if (isLess) {
            self.flags[2] = 1;
        } else {
            self.flags[2] = 0;
        }

        if (isEqu) {
            self.flags[1] = 1;
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
    }

    fn op_usqrt(&mut self) {
        // 0x17, size: 3
        // Square root of Rs to Rd
        let reg_dest: usize = self.memory[(self.ip + 1)] as usize;
        let reg_src: usize = self.memory[(self.ip + 2)] as usize;

        let res: u64 = self.registers[reg_src].isqrt();
        self.registers[reg_dest] = res;
        self.reg_types[reg_dest] = RegTypes::uint64;

        if (res == 0) {
            self.flags[1] = 1; //zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_upow(&mut self) {
        // 0x18, size: 3
        // Rd = Rd ** Rs
        let reg_dest: usize = self.memory[(self.ip + 1)] as usize;
        let reg_src: usize = self.memory[(self.ip + 2)] as usize;

        let res: u64 = self.registers[reg_dest].pow(self.registers[reg_src] as u32);
        self.registers[reg_dest] = res;
        if (res == 0) {
            self.flags[1] = 1; //zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_iload(&mut self) {
        //0x20, size: 10
        let register_ind: u8 = self.memory[(self.ip + 1) as usize];
        let value: i64 = args_to_i64(&self.memory[(self.ip + 2)..(self.ip + 10)]);

        self.registers[register_ind as usize] = value as u64;
        self.reg_types[register_ind as usize] = RegTypes::int64;

        self.ip += 10;
        return;
    }

    fn op_iadd(&mut self) {
        //0x21, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res: i64 = (self.registers[dest_r_ind as usize] as i64)
            + (self.registers[src_r_ind as usize] as i64);
        self.registers[dest_r_ind as usize] = res as u64;

        self.ip += 3;
        return;
    }

    fn op_imul(&mut self) {
        //0x22, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res: i64 = (self.registers[dest_r_ind as usize] as i64)
            * (self.registers[src_r_ind as usize] as i64);
        self.registers[dest_r_ind as usize] = res as u64;

        self.ip += 3;
        return;
    }

    fn op_isub(&mut self) {
        //0x23, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res: i64 = (self.registers[dest_r_ind as usize] as i64)
            - (self.registers[src_r_ind as usize] as i64);
        self.registers[dest_r_ind as usize] = res as u64;

        self.ip += 3;
        return;
    }

    fn op_idiv(&mut self) {
        //0x24, size: 4
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let reg_1: u8 = self.memory[(self.ip + 2) as usize];
        let reg_2: u8 = self.memory[(self.ip + 3) as usize];

        if (self.registers[reg_2 as usize] == 0) {
            panic!("DIVZERO exception at {}", self.ip);
        }
        let res: i64 =
            (self.registers[reg_1 as usize] as i64) / (self.registers[reg_2 as usize] as i64);
        self.registers[dest_r_ind as usize] = res as u64;

        self.reg_types[dest_r_ind as usize] = RegTypes::int64;

        self.ip += 4;
        return;
    }

    fn op_irem(&mut self) {
        //0x25, size: 4
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let reg_1: u8 = self.memory[(self.ip + 2) as usize];
        let reg_2: u8 = self.memory[(self.ip + 3) as usize];

        if (self.registers[reg_2 as usize] == 0) {
            panic!("DIVZERO exception at {}", self.ip);
        }
        let res: i64 =
            (self.registers[reg_1 as usize] as i64) % (self.registers[reg_2 as usize] as i64);
        self.registers[dest_r_ind as usize] = res as u64;

        self.reg_types[dest_r_ind as usize] = RegTypes::int64;

        self.ip += 4;
        return;
    }

    fn op_icmp(&mut self) {
        // 0x26, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let isLess: bool = ((self.registers[dest_r_ind as usize] as i64)
            < (self.registers[src_r_ind as usize] as i64));
        let isEqu: bool = ((self.registers[dest_r_ind as usize] as i64)
            == (self.registers[src_r_ind as usize] as i64));

        if (isLess) {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }
        if (isEqu) {
            self.flags[1] = 1;
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_iabs(&mut self) {
        // 0x27, size: 3
        // Save Absolute value of R src into R dest (Rd = abs(Rs))
        let reg_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let reg_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: u64 = (i64::abs(self.registers[reg_src_ind] as i64)) as u64;
        self.registers[reg_dest_ind] = res;
        self.reg_types[reg_dest_ind] = RegTypes::int64;

        if (res == 0) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_ineg(&mut self) {
        // 0x28, size: 3
        // Set R dest to arithmetically inverted R src
        let reg_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let reg_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: u64 = (-(self.registers[reg_src_ind] as i64)) as u64;
        self.registers[reg_dest_ind] = res;
        self.reg_types[reg_dest_ind] = RegTypes::int64;

        if (res == 0) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }
        if ((res as i64) < 0) {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_isqrt(&mut self) {
        //0x29, size: 3
        // INT64 square root
        let reg_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let reg_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: u64 = (self.registers[reg_src_ind] as i64).isqrt() as u64;
        self.registers[reg_dest_ind] = res;
        self.reg_types[reg_dest_ind] = RegTypes::int64;

        if (res == 0) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_ipow(&mut self) {
        //0x2a, size: 3
        // INT64 power (Rd = Rd ** Rs)
        let reg_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let reg_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: u64 =
            ((self.registers[reg_dest_ind] as i64).pow(self.registers[reg_src_ind] as u32)) as u64;
        self.registers[reg_dest_ind] = res;

        if (res == 0) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }
        if ((res as i64) < 0) {
            self.flags[2] = 1;
        } else {
            self.flags[2] = 0;
        }
        self.ip += 3;
        return;
    }

    fn op_fload(&mut self) {
        // 0x30, size: 10
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let float_val: f64 =
            args_to_f64(&self.memory[((self.ip + 2) as usize)..((self.ip + 10) as usize)]);

        self.registers[dest_r_ind as usize] = float_val.to_bits();
        self.reg_types[dest_r_ind as usize] = RegTypes::float64;

        self.ip += 10;
        return;
    }

    fn op_fadd(&mut self) {
        // 0x31, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let result: f64 = f64::from_bits(self.registers[dest_r_ind as usize])
            + f64::from_bits(self.registers[src_r_ind as usize]);
        self.registers[dest_r_ind as usize] = result.to_bits();

        self.ip += 3;
        return;
    }

    fn op_fmul(&mut self) {
        // 0x32, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let result: f64 = f64::from_bits(self.registers[dest_r_ind as usize])
            * f64::from_bits(self.registers[src_r_ind as usize]);
        self.registers[dest_r_ind as usize] = result.to_bits();

        self.ip += 3;
        return;
    }

    fn op_fsub(&mut self) {
        // 0x33, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let result: f64 = f64::from_bits(self.registers[dest_r_ind as usize])
            - f64::from_bits(self.registers[src_r_ind as usize]);
        self.registers[dest_r_ind as usize] = result.to_bits();

        self.ip += 3;
        return;
    }

    fn op_fdiv(&mut self) {
        // 0x34, size: 4
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let reg_1_ind: u8 = self.memory[(self.ip + 2) as usize];
        let reg_2_ind: u8 = self.memory[(self.ip + 3) as usize];

        let result: f64 = f64::from_bits(self.registers[reg_1_ind as usize])
            / f64::from_bits(self.registers[reg_2_ind as usize]);
        self.registers[dest_r_ind as usize] = result.to_bits();
        self.reg_types[dest_r_ind as usize] = RegTypes::float64;

        self.ip += 4;
        return;
    }

    fn op_frem(&mut self) {
        // 0x35, size: 4
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let reg_1_ind: u8 = self.memory[(self.ip + 2) as usize];
        let reg_2_ind: u8 = self.memory[(self.ip + 3) as usize];

        let result: f64 = f64::from_bits(self.registers[reg_1_ind as usize])
            % f64::from_bits(self.registers[reg_2_ind as usize]);
        self.registers[dest_r_ind as usize] = result.to_bits();
        self.reg_types[dest_r_ind as usize] = RegTypes::float64;

        self.ip += 4;
        return;
    }

    fn op_fcmp(&mut self) {
        // 0x36, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let isLess: bool = ((f64::from_bits(self.registers[dest_r_ind as usize]))
            < (f64::from_bits(self.registers[src_r_ind as usize])));
        let isEqu: bool = ((f64::from_bits(self.registers[dest_r_ind as usize]))
            == (f64::from_bits(self.registers[src_r_ind as usize])));

        if (isLess) {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }
        if (isEqu) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_fcmp_eps(&mut self) {
        // 0x37, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let dest_val: f64 = f64::from_bits(self.registers[dest_r_ind as usize]);
        let src_val: f64 = f64::from_bits(self.registers[src_r_ind as usize]);
        let epsilon: f64 = self.float_epsilon;

        let isLess: bool = ((src_val - dest_val) > (epsilon));
        let isEqu: bool = ((dest_val - src_val).abs() < (epsilon));

        if (isLess) {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }
        if (isEqu) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_fabs(&mut self) {
        // 0x38, size: 3
        // Save Absolute value of R src into R dest (Rd = abs(Rs))
        let reg_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let reg_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: f64 = f64::abs(f64::from_bits(self.registers[reg_src_ind]));
        self.registers[reg_dest_ind] = res.to_bits();
        self.reg_types[reg_dest_ind] = RegTypes::float64;

        if (res == 0.0f64) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_fneg(&mut self) {
        // 0x39, size: 3
        // Arithmetical inversion of float64 Rs. Save into Rd
        let reg_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let reg_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: f64 = -(f64::from_bits(self.registers[reg_src_ind]));
        self.registers[reg_dest_ind] = res.to_bits();
        self.reg_types[reg_dest_ind] = RegTypes::float64;

        if (res == 0.0f64) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }
        if (res < 0.0f64) {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_fsqrt(&mut self) {
        // 0x3a, size: 3
        // Save the square root of Rs into Rd
        let reg_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let reg_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: f64 = f64::from_bits(self.registers[reg_src_ind]).sqrt();
        self.registers[reg_dest_ind] = res.to_bits();
        self.reg_types[reg_dest_ind] = RegTypes::float64;

        if (res == 0.0f64) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_fpow(&mut self) {
        // 0x3b, size: 3
        // Rd = Rd ** Rs
        let reg_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let reg_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: f64 = f64::from_bits(self.registers[reg_dest_ind])
            .powf(f64::from_bits(self.registers[reg_src_ind]));
        self.registers[reg_dest_ind] = res.to_bits();
        self.reg_types[reg_dest_ind] = RegTypes::float64;

        if (res == 0.0f64) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_jmp(&mut self) {
        // 0x40, size: 9
        let target_addr: u64 = args_to_u64(&self.memory[(self.ip + 1)..(self.ip + 9)]);
        self.ip = target_addr as usize;
        return;
    }

    fn op_jz(&mut self) {
        // 0x41, size: 9
        //println!("DBG: JZ, ZF = {}", self.flags[1]);
        if (self.flags[1] != 0) {
            let target_addr: u64 = args_to_u64(&self.memory[(self.ip + 1)..(self.ip + 9)]);
            self.ip = target_addr as usize;
            return;
        } else {
            self.ip += 9;
            return;
        }
    }

    fn op_jl(&mut self) {
        // 0x42, size: 9
        //println!("DBG: NF = {}", self.flags[2]);
        if (self.flags[2] != 0) {
            let target_addr: u64 = args_to_u64(&self.memory[(self.ip + 1)..(self.ip + 9)]);
            self.ip = target_addr as usize;
            return;
        } else {
            self.ip += 9;
            return;
        }
    }

    fn op_jg(&mut self) {
        // 0x43, size: 9
        if (self.flags[1] == 0) && (self.flags[2] == 0) {
            let target_addr: u64 = args_to_u64(&self.memory[(self.ip + 1)..(self.ip + 9)]);
            self.ip = target_addr as usize;
            return;
        } else {
            self.ip += 9;
            return;
        }
    }

    fn op_jge(&mut self) {
        // 0x44, size: 9
        if (self.flags[2] == 0) {
            let target_addr: u64 = args_to_u64(&self.memory[(self.ip + 1)..(self.ip + 9)]);
            self.ip = target_addr as usize;
            return;
        } else {
            self.ip += 9;
            return;
        }
    }

    fn op_jle(&mut self) {
        // 0x45, size: 9
        if (self.flags[2] == 1) || (self.flags[1] == 1) {
            let target_addr: u64 = args_to_u64(&self.memory[(self.ip + 1)..(self.ip + 9)]);
            self.ip = target_addr as usize;
            return;
        } else {
            self.ip += 9;
            return;
        }
    }

    fn op_utoi(&mut self) {
        // 0x50, size: 3
        // Transfers unsigned integer UINT64 into signed integer INT64
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: i64 = self.registers[r_src_ind as usize] as i64;
        self.registers[r_dest_ind as usize] = res_val as u64;
        self.reg_types[r_dest_ind as usize] = RegTypes::int64;

        self.ip += 3;
        return;
    }

    fn op_itou(&mut self) {
        // 0x51, size: 3
        // Transfers signed integer (int64) int unsigned integer (uint64)
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: u64 = (self.registers[r_src_ind as usize] as i64).abs() as u64;

        self.registers[r_dest_ind as usize] = res_val;
        self.reg_types[r_dest_ind as usize] = RegTypes::uint64;

        self.ip += 3;
        return;
    }

    fn op_utof(&mut self) {
        // 0x52, size: 3
        // Transfers unsigned integer UINT64 into floating point value float64
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: f64 = self.registers[r_src_ind as usize] as f64;

        self.registers[r_dest_ind as usize] = res_val.to_bits();
        self.reg_types[r_dest_ind as usize] = RegTypes::float64;

        self.ip += 3;
        return;
    }

    fn op_itof(&mut self) {
        // 0x53, size: 3
        // Transfers signed integer INT64 into floating point value float64
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: f64 = (self.registers[r_src_ind as usize] as i64) as f64;

        self.registers[r_dest_ind as usize] = res_val.to_bits();
        self.reg_types[r_dest_ind as usize] = RegTypes::float64;

        self.ip += 3;
        return;
    }

    fn op_ftou(&mut self) {
        // 0x54, size: 3
        // Transfers floating point value FLOAT64 into unsigned integer value UINT64
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: u64 = f64::from_bits(self.registers[r_src_ind as usize]) as u64;

        self.registers[r_dest_ind as usize] = res_val;
        self.reg_types[r_dest_ind as usize] = RegTypes::uint64;

        self.ip += 3;
        return;
    }

    fn op_ftoi(&mut self) {
        // 0x55, size: 3
        // Transfers floating point value FLOAT64 into signed integer INT64
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: i64 = f64::from_bits(self.registers[r_src_ind as usize]) as i64;

        self.registers[r_dest_ind as usize] = res_val as u64;
        self.reg_types[r_dest_ind as usize] = RegTypes::int64;

        self.ip += 3;
        return;
    }

    fn op_movr(&mut self) {
        // 0x60, size: 3
        // Copies value of R src into R dest, saving the type.
        let r_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        self.registers[r_dest_ind as usize] = self.registers[r_src_ind as usize];
        self.reg_types[r_dest_ind as usize] = self.reg_types[r_src_ind as usize];

        self.ip += 3;
        return;
    }

    fn op_or(&mut self) {
        // 0x61, size: 3
        // Bitwise OR of R dest and R src, save into R dest
        // Basically: Rd = Rd | Rs
        let r_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: u64 = self.registers[r_dest_ind] | self.registers[r_src_ind];
        self.registers[r_dest_ind] = res;
        self.reg_types[r_dest_ind] = self.reg_types[r_src_ind];
        if (res == 0) {
            self.flags[1] = 1;
        } else {
            self.flags[0] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_and(&mut self) {
        // 0x62, size: 3
        // Bitwise AND of R dest and R src, save into R dest
        // Basically: Rd = Rd & Rs
        let r_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: u64 = self.registers[r_dest_ind] & self.registers[r_src_ind];
        self.registers[r_dest_ind] = res;
        self.reg_types[r_dest_ind] = self.reg_types[r_src_ind];
        if (res == 0) {
            self.flags[1] = 1;
        } else {
            self.flags[0] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_not(&mut self) {
        // 0x63, size: 3
        // Bitwise inversion of R src, save into R dest
        // Basically: Rd = ~Rs
        let r_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: u64 = !self.registers[r_src_ind];
        self.registers[r_dest_ind] = res;
        self.reg_types[r_dest_ind] = self.reg_types[r_src_ind];
        if (res == 0) {
            self.flags[1] = 1;
        } else {
            self.flags[0] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_xor(&mut self) {
        // 0x64, size: 3
        // Bitwise XOR (exclusive OR) of R dest and R src, save into R dest
        // Basically: Rd = Rd ^ Rs
        let r_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: u64 = self.registers[r_dest_ind] ^ self.registers[r_src_ind];
        self.registers[r_dest_ind] = res;
        self.reg_types[r_dest_ind] = self.reg_types[r_src_ind];
        if (res == 0) {
            self.flags[1] = 1;
        } else {
            self.flags[0] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_test(&mut self) {
        // 0x65, size: 3
        // Bitwise AND of R dest and R src, but without saving the result
        // Basically: Rd & Rs, change ZF.
        let r_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: u64 = self.registers[r_dest_ind] & self.registers[r_src_ind];
        if (res == 0) {
            self.flags[1] = 1;
        } else {
            self.flags[0] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_lnot(&mut self) {
        // 0x66, size: 3
        // Performs logical inversion (for booleans) of R src, saves into R dest
        // Basically: R dest = !Rs
        let r_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        let res: u64 = if (self.registers[r_src_ind] == 0) {
            1
        } else {
            0
        };
        self.registers[r_dest_ind] = res;
        self.reg_types[r_dest_ind] = self.reg_types[r_src_ind];
        if (res == 0) {
            self.flags[1] = 1;
        } else {
            self.flags[0] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_dsload(&mut self) {
        // 0x70, size: 18
        // dsload Rdest reladdr offset
        let rel_addr: usize =
            args_to_u64(&self.memory[(self.ip + 2 as usize)..(self.ip + 10 as usize)]) as usize; // relative address of target variable in VM memory
        let offset: usize =
            args_to_u64(&self.memory[(self.ip + 10 as usize)..(self.ip + 18 as usize)]) as usize;
        /*println!(
            "DBG Parsed offset: {}, parsed rel addr: {}, data base: {}",
            offset, rel_addr, self.data_base
        );*/
        let abs_addr: usize = (self.data_base as usize) + rel_addr + offset; // absolute addr
        let var_type_ind: u8 = self.memory[abs_addr - offset];
        let var_type: RegTypes = match var_type_ind {
            0x1 => RegTypes::uint64,
            0x2 => RegTypes::int64,
            0x3 => RegTypes::float64,
            0x4 => RegTypes::StrAddr,
            other => panic!(
                "CRITICAL: Unknown constant type: {}. IP: {}",
                other, self.ip
            ),
        };
        match var_type {
            RegTypes::uint64 => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] =
                    args_to_u64(&self.memory[(abs_addr + 1)..(abs_addr + 9)]);
                self.reg_types[dest_reg_ind as usize] = RegTypes::uint64;
                //println!("DBG start addr: {}", abs_addr + 2);
            }
            RegTypes::int64 => {
                let res: i64 = args_to_i64(&self.memory[(abs_addr + 1)..(abs_addr + 9)]);
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] = res as u64;
                self.reg_types[dest_reg_ind as usize] = RegTypes::int64;
            }
            RegTypes::float64 => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                let res: f64 = args_to_f64(&self.memory[(abs_addr + 1)..(abs_addr + 9)]);
                self.registers[dest_reg_ind as usize] = res.to_bits();
                self.reg_types[dest_reg_ind as usize] = RegTypes::float64;
            }
            RegTypes::StrAddr => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] = (abs_addr + 8 + 1) as u64; // +1 for type, +8 for length
                self.reg_types[dest_reg_ind as usize] = RegTypes::StrAddr;
            }
            RegTypes::address => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] = (abs_addr + 8 + 1) as u64; // +1 for type, +8 for length
                self.reg_types[dest_reg_ind as usize] = RegTypes::address;
            }
        }

        self.ip += 18;
        return;
    }

    fn ncall_println(&mut self) {
        // size: 4
        let src_r_num: u8 = self.memory[(self.ip + 3)];
        match self.reg_types[src_r_num as usize] {
            RegTypes::uint64 => {
                println!("{}", self.registers[src_r_num as usize]);
            }
            RegTypes::int64 => {
                println!("{}", self.registers[src_r_num as usize] as i64);
            }
            RegTypes::float64 => {
                let val: f64 = f64::from_bits(self.registers[src_r_num as usize]);
                println!("{}", format_float(val));
            }
            RegTypes::address => {
                // TODO: Implement printing data from addr
            }
            RegTypes::StrAddr => {
                //let rel_addr: u64 = self.registers[src_r_num as usize];
                //let abs_addr: u64 = self.data_base + rel_addr;
                let abs_addr: u64 = self.registers[src_r_num as usize];
                let bytes_len = &self.memory[((abs_addr - 8) as usize)..((abs_addr) as usize)];
                let size: u64 = u64::from_be_bytes(bytes_len.try_into().unwrap());

                let bytes_str = &self.memory[(abs_addr as usize)..((abs_addr + size) as usize)];
                let utf16_data = u8_slice_to_u16_vec(bytes_str);

                let res_str: String = match String::from_utf16(&utf16_data) {
                    Ok(val) => val,
                    Err(err) => panic!(
                        "CRITICAL: While converting into utf8 printable string: {}",
                        err
                    ),
                };
                println!("{}", res_str);
            }
        }
        self.ip += 4;
        return;
    }
}

pub fn args_to_u64(args: &[u8]) -> u64 {
    let bytes: [u8; 8] = args.try_into().expect(&format!("Bytes convertion error!"));
    let value: u64 = u64::from_be_bytes(bytes);
    value
}

pub fn args_to_u16(args: &[u8]) -> u16 {
    let bytes: [u8; 2] = args.try_into().expect(&format!("Bytes convertion error!"));
    let value: u16 = u16::from_be_bytes(bytes);
    value
}

pub fn args_to_i64(args: &[u8]) -> i64 {
    let bytes: [u8; 8] = args.try_into().expect(&format!("Bytes convertion error!"));
    let value: i64 = i64::from_be_bytes(bytes);
    value
}

pub fn args_to_f64(args: &[u8]) -> f64 {
    let bytes: [u8; 8] = args
        .try_into()
        .expect(&format!("Bytes convertion error into f64!"));
    let value: f64 = f64::from_be_bytes(bytes);
    value
}

pub fn format_float(value: f64) -> String {
    let s = format!("{:.11}", value);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if (s.is_empty()) {
        "0".to_string()
    } else {
        s.to_string()
    }
}

pub fn u8_slice_to_u16_vec(bytes: &[u8]) -> Vec<u16> {
    bytes
        .chunks(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect()
}
