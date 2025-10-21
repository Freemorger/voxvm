#![allow(non_snake_case)]

use crate::{
    callstack::CallStack,
    exceptions::Exception,
    fileformats::VoxExeHeader,
    func_ops::{op_call, op_callr, op_fnstind, op_ret},
    gc::GC,
    heap::{Heap, op_alloc, op_allocr, op_allocr_nogc, op_free, op_load, op_store},
    native::NativeService,
    registers::Register,
    stack::{VMStack, op_gsf, op_pop, op_popall, op_push, op_pushall, op_usf},
};
use core::panic;
use std::{
    collections::{HashMap, HashSet},
    fmt::Result,
    fs::{self, File},
    io::Write,
    time::Instant,
};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
pub enum RegTypes {
    uint64 = 1,
    int64 = 2,
    float64 = 3,
    StrAddr = 4,
    address = 8,
    ds_addr = 9,
}

#[derive(Debug)]
pub struct VM {
    pub registers: [Register; 32],
    pub reg_types: [RegTypes; 32],
    flags: [u8; 4], // of, zf, nf, cf
    pub ip: usize,
    pub memory: Vec<u8>, // dividing by each bytes, then can be grouped
    pub stack: VMStack,
    pub heap: Heap,
    pub gc: GC,
    data_base: u64, // pointer at data segment start
    data_size: u64,
    pub nativesys: NativeService,
    running: bool,
    float_epsilon: f64,
    pub func_table: Vec<u64>,
    pub call_stack: CallStack,
    pub rec_depth_max: usize,
    pub exceptions_active: Vec<Exception>,
}
type NativeFn = fn(&mut VM, &[u64]) -> Result;
type InstructionHandler = fn(&mut VM);

impl VM {
    pub fn new(
        init_mem: usize,
        init_stack: usize,
        init_heap: usize,
        max_recursion_depth: usize,
    ) -> VM {
        VM {
            registers: [Register::uint(0); 32],
            reg_types: [RegTypes::uint64; 32],
            flags: [0; 4],
            ip: 0x0,
            memory: Vec::with_capacity(init_mem),
            stack: VMStack::new(init_stack),
            heap: Heap::new(init_heap),
            data_base: 0x0,
            data_size: 0,
            nativesys: NativeService::new(),
            running: true,
            float_epsilon: 1e-10,
            func_table: Vec::new(),
            call_stack: CallStack::new(),
            rec_depth_max: max_recursion_depth,
            exceptions_active: Vec::new(),
            gc: GC::new(),
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

        let header_size: usize = (0x30 + fileHeader.func_table_len * 16) as usize;
        self.ip = fileHeader.entry_point as usize;
        self.data_base = fileHeader.data_base;
        self.data_size = fileHeader.data_size;
        self.func_table = fileHeader.func_table.clone();

        match fs::read(input_file_name) {
            Ok(bytes) => {
                for byte in &bytes[header_size..] {
                    self.memory.push(*byte);
                }
            }
            Err(err) => {
                panic!("CRITICAL: Can't read .vve file. Error: {}", err)
            }
        }
    }

    pub fn run(&mut self) {
        let mut since_cleanup: usize = 0;

        let run_start = Instant::now();
        while (self.ip < self.memory.capacity()) && (self.running) {
            let opcode = self.memory[self.ip];
            //println!("DBG: cur opcode: {}", self.ip);
            Self::OPERATIONS[opcode as usize](self);

            if (since_cleanup >= 250) {
                // running gc after each 250 instructions
                let start = Instant::now();

                let regs_hashset: HashSet<u64> = self.gc_gen_reg_set();
                let dstack_hashset: HashSet<u64> = self.fetch_dstack_refs();
                let final_hset: HashSet<u64> =
                    regs_hashset.union(&dstack_hashset).cloned().collect();
                let t2: HashMap<u64, HashSet<u64>> = self.heap.saved_refs.clone();

                self.gc.mark(&final_hset, &t2);
                let addrs = self.gc.sweep();
                self.gc_finish_cleanup(addrs);

                let elapsed = start.elapsed();

                //println!("elapsed on gc: {:?}", elapsed);
                since_cleanup = 0;
            } else {
                since_cleanup += 1;
            }
        }
        if self.ip >= self.memory.capacity() {
            panic!(
                "CRITICAL: Instruction overflow! VM Memory capacity: {}, latest opcode: {}.
                \n Consider running VM with more init ram using
                --init-ram=RAM_VALUE",
                self.memory.capacity(),
                self.ip
            );
        }
        let end_run = run_start.elapsed();
        //println!("Elapsed on end_run: {:?}", end_run);
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
        handlers[0x19] = Self::op_uinc as InstructionHandler;
        handlers[0x1a] = Self::op_udec as InstructionHandler;
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
        handlers[0x2b] = Self::op_iinc as InstructionHandler;
        handlers[0x2c] = Self::op_idec as InstructionHandler;
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
        handlers[0x3c] = Self::op_finc as InstructionHandler;
        handlers[0x3d] = Self::op_fdec as InstructionHandler;
        handlers[0x40] = Self::op_jmp as InstructionHandler;
        handlers[0x41] = Self::op_jz as InstructionHandler;
        handlers[0x42] = Self::op_jl as InstructionHandler;
        handlers[0x43] = Self::op_jg as InstructionHandler;
        handlers[0x44] = Self::op_jge as InstructionHandler;
        handlers[0x45] = Self::op_jle as InstructionHandler;
        handlers[0x46] = Self::op_jexc as InstructionHandler;
        handlers[0x50] = Self::op_utoi as InstructionHandler;
        handlers[0x51] = Self::op_itou as InstructionHandler;
        handlers[0x52] = Self::op_utof as InstructionHandler;
        handlers[0x53] = Self::op_itof as InstructionHandler;
        handlers[0x54] = Self::op_ftou as InstructionHandler;
        handlers[0x55] = Self::op_ftoi as InstructionHandler;
        handlers[0x56] = Self::op_ptou as InstructionHandler;
        handlers[0x57] = Self::op_utop as InstructionHandler;
        handlers[0x60] = Self::op_movr as InstructionHandler;
        handlers[0x61] = Self::op_or as InstructionHandler;
        handlers[0x62] = Self::op_and as InstructionHandler;
        handlers[0x63] = Self::op_not as InstructionHandler;
        handlers[0x64] = Self::op_xor as InstructionHandler;
        handlers[0x65] = Self::op_test as InstructionHandler;
        handlers[0x66] = Self::op_lnot as InstructionHandler;
        handlers[0x70] = Self::op_dsload as InstructionHandler;
        handlers[0x71] = Self::op_dsrload as InstructionHandler;
        handlers[0x72] = Self::op_dssave as InstructionHandler;
        handlers[0x73] = Self::op_dsrsave as InstructionHandler;
        handlers[0x74] = Self::op_dslea as InstructionHandler;
        handlers[0x75] = Self::op_dsderef as InstructionHandler;
        handlers[0x76] = Self::op_dsrlea as InstructionHandler;
        handlers[0x77] = Self::op_dsrderef as InstructionHandler;
        handlers[0x80] = op_push as InstructionHandler;
        handlers[0x81] = op_pop as InstructionHandler;
        handlers[0x82] = op_pushall as InstructionHandler;
        handlers[0x83] = op_popall as InstructionHandler;
        handlers[0x84] = op_gsf as InstructionHandler;
        handlers[0x85] = op_usf as InstructionHandler;
        handlers[0x90] = op_call as InstructionHandler;
        handlers[0x91] = op_ret as InstructionHandler;
        handlers[0x92] = op_fnstind as InstructionHandler;
        handlers[0x93] = op_callr as InstructionHandler;
        handlers[0xA0] = op_alloc as InstructionHandler;
        handlers[0xA1] = op_free as InstructionHandler;
        handlers[0xA2] = op_store as InstructionHandler;
        handlers[0xA3] = op_allocr as InstructionHandler;
        handlers[0xA4] = op_load as InstructionHandler;
        handlers[0xA5] = op_allocr_nogc as InstructionHandler;
        // ...
        handlers
    };

    fn fetch_dstack_refs(&mut self) -> HashSet<u64> {
        let mut res: HashSet<u64> = HashSet::new();
        let len = self.stack.stack.len();
        if (len == 0) {
            return res;
        }

        for i in 0..(len.saturating_sub(1)) {
            if (self.stack.stack[i].ftype == RegTypes::address) {
                res.insert(self.stack.stack[i].val);
            }
        }
        res
    }

    fn gc_finish_cleanup(&mut self, ptrs: Vec<u64>) {
        for ptr in ptrs {
            match self.heap.free(ptr) {
                Ok(_) => {}
                Err(_) => {
                    println!(
                        "INFO: No object with ptr {:x} found in heap in GC cleanup",
                        ptr
                    );
                    self.gc.main_refs.remove(&ptr);
                }
            }
        }
    }

    fn gc_gen_reg_set(&mut self) -> HashSet<u64> {
        let mut res: HashSet<u64> = HashSet::new();
        for (idx, reg) in self.registers.iter().enumerate() {
            if self.reg_types[idx] == RegTypes::address {
                res.insert(reg.as_u64());
            }
        }
        res
    }

    fn op_unimplemented(&mut self) {
        //self.err_coredump();
        panic!(
            "CRITICAL: Unknown operation code at {:#x}: {:#x}.",
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
            other => {} //self.nativesys.call_code(other),
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

        self.registers[register_ind as usize] = Register::uint(value);
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
        if self.registers[in_reg_ind as usize] == Register::uint(0) {
            self.flags[1] = 1;
        } else {
            self.flags[1] = 0;
        }
        self.ip += 3;
        return;
    }

    fn op_udiv(&mut self) {
        // 0x14, size: 4
        let reg_out: u8 = self.memory[self.ip + 1];
        let reg_1: u8 = self.memory[self.ip + 2];
        let reg_2: u8 = self.memory[self.ip + 3];
        if self.registers[reg_2 as usize] == Register::uint(0) {
            eprintln!("DIVZERO Exception at addr {}", self.ip);
            self.exceptions_active.push(Exception::ZeroDivision);
        }

        self.registers[reg_out as usize] =
            self.registers[reg_1 as usize] / self.registers[reg_2 as usize];

        self.reg_types[reg_out as usize] = RegTypes::uint64;

        self.ip += 4;
    }

    fn op_urem(&mut self) {
        // 0x15, size: 4
        let reg_dest: u8 = self.memory[self.ip + 1];
        let reg_1: u8 = self.memory[self.ip + 2];
        let reg_2: u8 = self.memory[self.ip + 3];

        self.registers[reg_dest as usize] =
            self.registers[reg_1 as usize] % self.registers[reg_2 as usize];

        self.reg_types[reg_dest as usize] = RegTypes::uint64;

        self.ip += 4;
    }

    fn op_ucmp(&mut self) {
        // 0x16, size: 3
        let reg_dest: u8 = self.memory[self.ip + 1];
        let reg_src: u8 = self.memory[self.ip + 2];

        let isLess: bool = self.registers[reg_dest as usize] < self.registers[reg_src as usize];
        let isEqu: bool = self.registers[reg_dest as usize] == self.registers[reg_src as usize];

        if isLess {
            self.flags[2] = 1;
        } else {
            self.flags[2] = 0;
        }

        if isEqu {
            self.flags[1] = 1;
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
    }

    fn op_usqrt(&mut self) {
        // 0x17, size: 3
        // Square root of Rs to Rd
        let reg_dest: usize = self.memory[self.ip + 1] as usize;
        let reg_src: usize = self.memory[self.ip + 2] as usize;

        let res: u64 = self.registers[reg_src].as_u64().isqrt();
        self.registers[reg_dest] = Register::uint(res);
        self.reg_types[reg_dest] = RegTypes::uint64;

        if res == 0 {
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
        let reg_dest: usize = self.memory[self.ip + 1] as usize;
        let reg_src: usize = self.memory[self.ip + 2] as usize;

        let res: u64 = self.registers[reg_dest]
            .as_u64()
            .pow(self.registers[reg_src].as_u64() as u32);
        self.registers[reg_dest] = Register::uint(res);
        if res == 0 {
            self.flags[1] = 1; //zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_uinc(&mut self) {
        // 0x19, size: 2
        // uinc Rdest
        let r_dest_int: usize = self.memory[(self.ip + 1)] as usize;
        self.registers[r_dest_int] += Register::uint(1);
        if self.registers[r_dest_int] == Register::uint(0) {
            self.flags[0] = 1; // of
        } else {
            self.flags[0] = 0;
        }

        self.ip += 2;
        return;
    }

    fn op_udec(&mut self) {
        // 0x1a, size: 2
        // udec Rdest
        let r_dest_int: usize = self.memory[(self.ip + 1)] as usize;
        self.registers[r_dest_int] -= Register::uint(1);
        if self.registers[r_dest_int] == Register::uint(0) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 2;
        return;
    }

    fn op_iload(&mut self) {
        //0x20, size: 10
        let register_ind: u8 = self.memory[(self.ip + 1) as usize];
        let value: i64 = args_to_i64(&self.memory[(self.ip + 2)..(self.ip + 10)]);

        self.registers[register_ind as usize] = Register::int(value);
        self.reg_types[register_ind as usize] = RegTypes::int64;

        self.ip += 10;
        return;
    }

    fn op_iadd(&mut self) {
        //0x21, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res: Register =
            self.registers[dest_r_ind as usize] + self.registers[src_r_ind as usize];
        self.registers[dest_r_ind as usize] = res;

        self.ip += 3;
        return;
    }

    fn op_imul(&mut self) {
        //0x22, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res: Register =
            self.registers[dest_r_ind as usize] * self.registers[src_r_ind as usize];
        self.registers[dest_r_ind as usize] = res;

        self.ip += 3;
        return;
    }

    fn op_isub(&mut self) {
        //0x23, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res: Register =
            self.registers[dest_r_ind as usize] - self.registers[src_r_ind as usize];
        self.registers[dest_r_ind as usize] = res;

        self.ip += 3;
        return;
    }

    fn op_idiv(&mut self) {
        //0x24, size: 4
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let reg_1: u8 = self.memory[(self.ip + 2) as usize];
        let reg_2: u8 = self.memory[(self.ip + 3) as usize];

        if self.registers[reg_2 as usize] == Register::int(0) {
            panic!("DIVZERO exception at {}", self.ip);
        }
        let res: Register = self.registers[reg_1 as usize] / self.registers[reg_2 as usize];
        self.registers[dest_r_ind as usize] = res;

        self.reg_types[dest_r_ind as usize] = RegTypes::int64;

        self.ip += 4;
        return;
    }

    fn op_irem(&mut self) {
        //0x25, size: 4
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let reg_1: u8 = self.memory[(self.ip + 2) as usize];
        let reg_2: u8 = self.memory[(self.ip + 3) as usize];

        if self.registers[reg_2 as usize] == Register::int(0) {
            panic!("DIVZERO exception at {}", self.ip);
        }
        let res: Register = self.registers[reg_1 as usize] % self.registers[reg_2 as usize];
        self.registers[dest_r_ind as usize] = res;

        self.reg_types[dest_r_ind as usize] = RegTypes::int64;

        self.ip += 4;
        return;
    }

    fn op_icmp(&mut self) {
        // 0x26, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let isLess: bool = self.registers[dest_r_ind as usize] < self.registers[src_r_ind as usize];
        let isEqu: bool = self.registers[dest_r_ind as usize] == self.registers[src_r_ind as usize];

        if isLess {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }
        if isEqu {
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

        let res: i64 = i64::abs(self.registers[reg_src_ind].as_i64());
        self.registers[reg_dest_ind] = Register::int(res);
        self.reg_types[reg_dest_ind] = RegTypes::int64;

        if res == 0 {
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

        let res: Register = -(self.registers[reg_src_ind]);
        self.registers[reg_dest_ind] = res;
        self.reg_types[reg_dest_ind] = RegTypes::int64;

        if res == Register::int(0) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }
        if res < Register::int(0) {
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

        if self.registers[reg_src_ind] < Register::int(0) {
            self.exceptions_active.push(Exception::NegativeSqrt);
            self.ip += 3;
            return;
        }
        let res: i64 = self.registers[reg_src_ind].as_i64().isqrt();
        self.registers[reg_dest_ind] = Register::int(res);
        self.reg_types[reg_dest_ind] = RegTypes::int64;

        if res == 0 {
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

        let res: i64 = (self.registers[reg_dest_ind].as_i64())
            .pow(self.registers[reg_src_ind].as_i64() as u32);
        self.registers[reg_dest_ind] = Register::int(res);

        if res == 0 {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }
        if (res as i64) < 0 {
            self.flags[2] = 1;
        } else {
            self.flags[2] = 0;
        }
        self.ip += 3;
        return;
    }

    fn op_iinc(&mut self) {
        // 0x2b, size: 2
        // iinc rdst
        let r_dst_ind: usize = self.memory[(self.ip + 1)] as usize;

        let new_val: Register = self.registers[r_dst_ind] + Register::int(1);
        self.registers[r_dst_ind] = new_val;
        if (new_val == Register::int(0)) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }
        if (new_val < Register::int(0)) {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }

        self.ip += 2;
        return;
    }

    fn op_idec(&mut self) {
        // 0x2c, size: 2
        // idec rdst
        let r_dst_ind: usize = self.memory[(self.ip + 1)] as usize;

        let new_val: Register = self.registers[r_dst_ind] - Register::int(1);
        self.registers[r_dst_ind] = new_val;
        if (new_val == Register::int(0)) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }
        if (new_val < Register::int(0)) {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }

        self.ip += 2;
        return;
    }

    fn op_fload(&mut self) {
        // 0x30, size: 10
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let float_val: f64 =
            args_to_f64(&self.memory[((self.ip + 2) as usize)..((self.ip + 10) as usize)]);

        self.registers[dest_r_ind as usize] = Register::float(float_val);
        self.reg_types[dest_r_ind as usize] = RegTypes::float64;

        self.ip += 10;
        return;
    }

    fn op_fadd(&mut self) {
        // 0x31, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let result: Register =
            self.registers[dest_r_ind as usize] + self.registers[src_r_ind as usize];
        self.registers[dest_r_ind as usize] = result;

        self.ip += 3;
        return;
    }

    fn op_fmul(&mut self) {
        // 0x32, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let result: Register =
            self.registers[dest_r_ind as usize] * self.registers[src_r_ind as usize];
        self.registers[dest_r_ind as usize] = result;

        self.ip += 3;
        return;
    }

    fn op_fsub(&mut self) {
        // 0x33, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let result: Register =
            self.registers[dest_r_ind as usize] - self.registers[src_r_ind as usize];
        self.registers[dest_r_ind as usize] = result;

        self.ip += 3;
        return;
    }

    fn op_fdiv(&mut self) {
        // 0x34, size: 4
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let reg_1_ind: u8 = self.memory[(self.ip + 2) as usize];
        let reg_2_ind: u8 = self.memory[(self.ip + 3) as usize];

        if self.registers[reg_2_ind as usize] == Register::float(0f64) {
            self.exceptions_active.push(Exception::ZeroDivision);
            self.ip += 4;
            return;
        }
        let result: Register =
            self.registers[reg_1_ind as usize] / self.registers[reg_2_ind as usize];
        self.registers[dest_r_ind as usize] = result;
        self.reg_types[dest_r_ind as usize] = RegTypes::float64;

        self.ip += 4;
        return;
    }

    fn op_frem(&mut self) {
        // 0x35, size: 4
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let reg_1_ind: u8 = self.memory[(self.ip + 2) as usize];
        let reg_2_ind: u8 = self.memory[(self.ip + 3) as usize];

        let result: Register =
            self.registers[reg_1_ind as usize] % self.registers[reg_2_ind as usize];
        self.registers[dest_r_ind as usize] = result;
        self.reg_types[dest_r_ind as usize] = RegTypes::float64;

        self.ip += 4;
        return;
    }

    fn op_fcmp(&mut self) {
        // 0x36, size: 3
        let dest_r_ind: u8 = self.memory[(self.ip + 1) as usize];
        let src_r_ind: u8 = self.memory[(self.ip + 2) as usize];

        let isLess: bool = self.registers[dest_r_ind as usize] < self.registers[src_r_ind as usize];
        let isEqu: bool = self.registers[dest_r_ind as usize] == self.registers[src_r_ind as usize];

        if isLess {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }
        if isEqu {
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

        let dest_val: Register = self.registers[dest_r_ind as usize];
        let src_val: Register = self.registers[src_r_ind as usize];
        let epsilon: Register = Register::float(self.float_epsilon);

        let isLess: bool = (src_val - dest_val) > (epsilon);
        let isEqu: bool = (dest_val - src_val).as_f64().abs() < (epsilon.as_f64());

        if isLess {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }
        if isEqu {
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

        let res: f64 = f64::abs(self.registers[reg_src_ind].as_f64());
        self.registers[reg_dest_ind] = Register::float(res);
        self.reg_types[reg_dest_ind] = RegTypes::float64;

        if res == 0.0f64 {
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

        let res: Register = -(self.registers[reg_src_ind]);
        self.registers[reg_dest_ind] = res;
        self.reg_types[reg_dest_ind] = RegTypes::float64;

        if res == Register::float(0.0f64) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }
        if res < Register::float(0.0f64) {
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

        if self.registers[reg_src_ind] < Register::float(0.0f64) {
            self.exceptions_active.push(Exception::NegativeSqrt);
            self.ip += 3;
            return;
        }
        let res: f64 = self.registers[reg_src_ind].as_f64().sqrt();
        self.registers[reg_dest_ind] = Register::float(res);
        self.reg_types[reg_dest_ind] = RegTypes::float64;

        if res == 0.0f64 {
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

        let res: f64 = self.registers[reg_dest_ind]
            .as_f64()
            .powf(self.registers[reg_src_ind].as_f64());
        println!(
            "DBG {} ** {} = {}",
            self.registers[reg_dest_ind].as_f64(),
            self.registers[reg_src_ind].as_f64(),
            res
        );
        self.registers[reg_dest_ind] = Register::float(res);
        self.reg_types[reg_dest_ind] = RegTypes::float64;

        if res == 0.0f64 {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }

        self.ip += 3;
        return;
    }

    fn op_finc(&mut self) {
        // 0x3c, size: 2
        // finc rdst
        let r_dst_ind: usize = self.memory[(self.ip + 1)] as usize;
        let res: Register = self.registers[r_dst_ind] + Register::float(1f64);

        self.registers[r_dst_ind] = res;
        if res == Register::float(0.0f64) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }
        if res < Register::float(0.0f64) {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }

        self.ip += 2;
        return;
    }

    fn op_fdec(&mut self) {
        // 0x3d, size: 2
        // fdec rdst
        let r_dst_ind: usize = self.memory[(self.ip + 1)] as usize;
        let res: Register = self.registers[r_dst_ind] - Register::float(1f64);

        self.registers[r_dst_ind] = res;
        if res == Register::float(0.0f64) {
            self.flags[1] = 1; // zf
        } else {
            self.flags[1] = 0;
        }
        if res < Register::float(0.0f64) {
            self.flags[2] = 1; // nf
        } else {
            self.flags[2] = 0;
        }

        self.ip += 2;
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
        if self.flags[1] != 0 {
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
        if self.flags[2] != 0 {
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
        if self.flags[2] == 0 {
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

    fn op_jexc(&mut self) {
        // 0x46, size: 17
        // jexc exception_num addr
        // jumps at addr if exception was thrown
        let exc_n = args_to_u64(&self.memory[(self.ip + 1)..(self.ip + 9)]);
        let tojump = args_to_u64(&self.memory[(self.ip + 9)..(self.ip + 17)]);

        let exception: Exception = match exc_n {
            0x1 => Exception::ZeroDivision,
            0x2 => Exception::HeapAllocationFault,
            0x3 => Exception::HeapFreeFault,
            0x4 => Exception::HeapWriteFault,
            0x5 => Exception::HeapReadFault,
            0x6 => Exception::NegativeSqrt,
            other => {
                panic!("Unknown exception: {} at IP {}", other, self.ip);
            }
        };

        for (ind, ex) in self.exceptions_active.iter().enumerate() {
            if *ex == exception {
                self.ip = tojump as usize;
                self.exceptions_active.remove(ind);
                return;
            }
        }

        self.ip += 17;
        return;
    }

    fn op_utoi(&mut self) {
        // 0x50, size: 3
        // Transfers unsigned integer UINT64 into signed integer INT64
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: i64 = self.registers[r_src_ind as usize].as_u64() as i64;
        self.registers[r_dest_ind as usize] = Register::int(res_val);
        self.reg_types[r_dest_ind as usize] = RegTypes::int64;

        self.ip += 3;
        return;
    }

    fn op_itou(&mut self) {
        // 0x51, size: 3
        // Transfers signed integer (int64) int unsigned integer (uint64)
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: u64 = self.registers[r_src_ind as usize].as_i64() as u64;

        self.registers[r_dest_ind as usize] = Register::uint(res_val);
        self.reg_types[r_dest_ind as usize] = RegTypes::uint64;

        self.ip += 3;
        return;
    }

    fn op_utof(&mut self) {
        // 0x52, size: 3
        // Transfers unsigned integer UINT64 into floating point value float64
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: f64 = self.registers[r_src_ind as usize].as_u64() as f64;

        self.registers[r_dest_ind as usize] = Register::float(res_val);
        self.reg_types[r_dest_ind as usize] = RegTypes::float64;

        self.ip += 3;
        return;
    }

    fn op_itof(&mut self) {
        // 0x53, size: 3
        // Transfers signed integer INT64 into floating point value float64
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: f64 = self.registers[r_src_ind as usize].as_i64() as f64;

        self.registers[r_dest_ind as usize] = Register::float(res_val);
        self.reg_types[r_dest_ind as usize] = RegTypes::float64;

        self.ip += 3;
        return;
    }

    fn op_ftou(&mut self) {
        // 0x54, size: 3
        // Transfers floating point value FLOAT64 into unsigned integer value UINT64
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: u64 = self.registers[r_src_ind as usize].as_f64() as u64;

        self.registers[r_dest_ind as usize] = Register::uint(res_val);
        self.reg_types[r_dest_ind as usize] = RegTypes::uint64;

        self.ip += 3;
        return;
    }

    fn op_ftoi(&mut self) {
        // 0x55, size: 3
        // Transfers floating point value FLOAT64 into signed integer INT64
        let r_dest_ind: u8 = self.memory[(self.ip + 1) as usize];
        let r_src_ind: u8 = self.memory[(self.ip + 2) as usize];

        let res_val: i64 = self.registers[r_src_ind as usize].as_f64() as i64;

        self.registers[r_dest_ind as usize] = Register::int(res_val);
        self.reg_types[r_dest_ind as usize] = RegTypes::int64;

        self.ip += 3;
        return;
    }

    fn op_ptou(&mut self) {
        // 0x55, size: 3
        // ptou rdst rsrc
        // Pointer (Address) to uint. If the register the same,
        // Doesn't change the actual value, only tracked type
        let r_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        self.registers[r_dest_ind] = Register::uint(self.registers[r_src_ind].as_u64());
        self.reg_types[r_dest_ind] = RegTypes::uint64;

        self.ip += 3;
        return;
    }

    fn op_utop(&mut self) {
        // 0x57, size: 3
        // ptou rdst rsrc
        // uint to pointer (Address). If the register the same,
        // Doesn't change the actual value, only tracked type
        let r_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_src_ind: usize = self.memory[(self.ip + 2) as usize] as usize;

        self.registers[r_dest_ind] = Register::address(self.registers[r_src_ind].as_u64());
        self.reg_types[r_dest_ind] = RegTypes::address;

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

        let res: Register = self.registers[r_dest_ind] | self.registers[r_src_ind];
        self.registers[r_dest_ind] = res;
        self.reg_types[r_dest_ind] = self.reg_types[r_src_ind];
        if res.as_u64() == 0 {
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

        let res: Register = self.registers[r_dest_ind] & self.registers[r_src_ind];
        self.registers[r_dest_ind] = res;
        self.reg_types[r_dest_ind] = self.reg_types[r_src_ind];
        if res.as_u64() == 0 {
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

        let res: Register = !self.registers[r_src_ind];
        self.registers[r_dest_ind] = res;
        self.reg_types[r_dest_ind] = self.reg_types[r_src_ind];
        if res.as_u64() == 0 {
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

        let res: Register = self.registers[r_dest_ind] ^ self.registers[r_src_ind];
        self.registers[r_dest_ind] = res;
        self.reg_types[r_dest_ind] = self.reg_types[r_src_ind];
        if res.as_u64() == 0 {
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

        let res: Register = self.registers[r_dest_ind] & self.registers[r_src_ind];
        if res.as_u64() == 0 {
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

        let res: Register = self.registers[r_src_ind].logical_not();
        self.registers[r_dest_ind] = res;
        self.reg_types[r_dest_ind] = self.reg_types[r_src_ind];
        if res.as_u64() == 0 {
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
        const const_flag: u8 = 0x10;
        let rel_addr: usize =
            args_to_u64(&self.memory[(self.ip + 2 as usize)..(self.ip + 10 as usize)]) as usize; // relative address of target variable in VM memory
        let offset: usize =
            args_to_u64(&self.memory[(self.ip + 10 as usize)..(self.ip + 18 as usize)]) as usize
                + 8
                + 1; // 8 for length skip, 1 for type
        let abs_addr: usize = (self.data_base as usize) + rel_addr + offset; // absolute addr.
        let mut var_type_ind: u8 = self.memory[abs_addr - offset];
        var_type_ind = var_type_ind & !const_flag; // getting clear type
        if var_type_ind >= 0x6 && var_type_ind <= 0x8 {
            var_type_ind -= 5; // dsload only loading value. use dslea for loading addr
        }
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
                    Register::uint(args_to_u64(&self.memory[(abs_addr)..(abs_addr + 8)]));

                self.reg_types[dest_reg_ind as usize] = RegTypes::uint64;
            }
            RegTypes::int64 => {
                let res: i64 = args_to_i64(&self.memory[(abs_addr)..(abs_addr + 8)]);
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] = Register::int(res);
                self.reg_types[dest_reg_ind as usize] = RegTypes::int64;
            }
            RegTypes::float64 => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                let res: f64 = args_to_f64(&self.memory[(abs_addr)..(abs_addr + 8)]);
                self.registers[dest_reg_ind as usize] = Register::float(res);
                self.reg_types[dest_reg_ind as usize] = RegTypes::float64;
            }
            RegTypes::StrAddr => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] = Register::StrAddr(abs_addr as u64); // +1 for type, +8 for length
                self.reg_types[dest_reg_ind as usize] = RegTypes::StrAddr;
            }
            RegTypes::address => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] = Register::address(abs_addr as u64); // +1 for type, +8 for length
                self.reg_types[dest_reg_ind as usize] = RegTypes::address;
            }
            RegTypes::ds_addr => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] = Register::ds_addr(abs_addr as u64); // +1 for type, +8 for length
                self.reg_types[dest_reg_ind as usize] = RegTypes::ds_addr;
            }
        }

        self.ip += 18;
        return;
    }

    fn op_dsrload(&mut self) {
        // 0x71, size: 11
        // dsload Rdest Roffset reladdr
        const const_flag: u8 = 0x10;
        let offset: usize = (self.registers[self.memory[(self.ip + 2) as usize] as usize].as_u64()
            + 8
            + 1) as usize; // 8 for
        // length skip
        let rel_addr: usize =
            args_to_u64(&self.memory[(self.ip + 3 as usize)..(self.ip + 11 as usize)]) as usize; // relative address of target variable in VM memory
        let abs_addr: usize = (self.data_base as usize) + rel_addr + offset;
        let mut var_type_ind: u8 = self.memory[abs_addr - offset];
        var_type_ind = var_type_ind & !const_flag;
        if var_type_ind >= 0x6 && var_type_ind <= 0x8 {
            var_type_ind -= 5; // dsload only loading value. use dslea for loading addr
        }
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
                    Register::uint(args_to_u64(&self.memory[(abs_addr)..(abs_addr + 8)]));
                self.reg_types[dest_reg_ind as usize] = RegTypes::uint64;
                //println!("DBG start addr: {}", abs_addr + 2);
            }
            RegTypes::int64 => {
                let res: i64 = args_to_i64(&self.memory[(abs_addr)..(abs_addr + 8)]);
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] = Register::int(res);
                self.reg_types[dest_reg_ind as usize] = RegTypes::int64;
            }
            RegTypes::float64 => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                let res: f64 = args_to_f64(&self.memory[(abs_addr)..(abs_addr + 8)]);
                self.registers[dest_reg_ind as usize] = Register::float(res);
                self.reg_types[dest_reg_ind as usize] = RegTypes::float64;
            }
            RegTypes::StrAddr => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] = Register::StrAddr(abs_addr as u64); // +1 for type, +8 for length
                self.reg_types[dest_reg_ind as usize] = RegTypes::StrAddr;
            }
            RegTypes::address => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] = Register::address(abs_addr as u64); // +1 for type, +8 for length
                self.reg_types[dest_reg_ind as usize] = RegTypes::address;
            }
            RegTypes::ds_addr => {
                let dest_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
                self.registers[dest_reg_ind as usize] = Register::ds_addr(abs_addr as u64); // +1 for type, +8 for length
                self.reg_types[dest_reg_ind as usize] = RegTypes::ds_addr;
            }
        }

        self.ip += 11;
        return;
    }

    fn op_dssave(&mut self) {
        // 0x72, size: 18
        // dssave Rsrc rel_addr offset
        // Updates the value in data segment
        const CONST_MASK: u8 = 0x10;
        let r_src_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let rel_addr: usize = args_to_u64(&self.memory[(self.ip + 2)..(self.ip + 10)]) as usize;
        let offset: usize = args_to_u64(&self.memory[(self.ip + 10)..(self.ip + 18)]) as usize;

        let abs_addr: usize = (self.data_base as usize) + rel_addr + offset + 1 + 8; // +1 for var
        if (self.memory[self.data_base as usize + rel_addr] & CONST_MASK) != 0 {
            panic!(
                "CRITICAL: Attempting to write new value into DS constant at IP {}",
                self.ip
            );
        }
        // type, +1 for var size
        match self.reg_types[r_src_ind] {
            RegTypes::uint64 | RegTypes::StrAddr | RegTypes::address | RegTypes::ds_addr => {
                let val: [u8; 8] = self.registers[r_src_ind].as_u64().to_be_bytes();
                for i in 0..8 {
                    self.memory[abs_addr + i] = val[i];
                }
            }
            RegTypes::int64 => {
                let val: [u8; 8] = self.registers[r_src_ind].as_i64().to_be_bytes();
                for i in 0..8 {
                    self.memory[abs_addr + i] = val[i];
                }
            }
            RegTypes::float64 => {
                let val: [u8; 8] = (self.registers[r_src_ind].as_f64()).to_be_bytes();
                for i in 0..8 {
                    self.memory[abs_addr + i] = val[i];
                }
            }
        }

        self.ip += 18;
        return;
    }
    fn op_dsrsave(&mut self) {
        // 0x73, size: 11
        // dsrsave Rsrc Roffset rel_addr
        const CONST_MASK: u8 = 0x10;
        let r_src_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_offset_ind: usize = self.memory[(self.ip + 2) as usize] as usize;
        let offset = self.registers[r_offset_ind];
        let rel_addr: usize = args_to_u64(&self.memory[(self.ip + 3)..(self.ip + 11)]) as usize;

        let abs_addr: usize =
            (self.data_base as usize) + rel_addr + (offset.as_u64() as usize) + 1 + 8; // +1 for var
        // type, +1 for var size
        if (self.memory[self.data_base as usize + rel_addr] & CONST_MASK) != 0 {
            panic!(
                "CRITICAL: Attempting to write new value into DS constant at IP {}",
                self.ip
            );
        }
        match self.reg_types[r_src_ind] {
            RegTypes::uint64 | RegTypes::StrAddr | RegTypes::address | RegTypes::ds_addr => {
                let val: [u8; 8] = self.registers[r_src_ind].as_u64().to_be_bytes();
                for i in 0..8 {
                    self.memory[abs_addr + i] = val[i];
                }
            }
            RegTypes::int64 => {
                let val: [u8; 8] = self.registers[r_src_ind].as_i64().to_be_bytes();
                for i in 0..8 {
                    self.memory[abs_addr + i] = val[i];
                }
            }
            RegTypes::float64 => {
                let val: [u8; 8] = self.registers[r_src_ind].as_f64().to_be_bytes();
                for i in 0..8 {
                    self.memory[abs_addr + i] = val[i];
                }
            }
        }

        self.ip += 11;
        return;
    }
    fn op_dslea(&mut self) {
        // 0x74, size: 18
        // dslea Rdest rel_addr offset
        let r_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let rel_addr: u64 =
            args_to_u64(&self.memory[(self.ip + 2) as usize..(self.ip + 10) as usize]);
        let offset: u64 =
            args_to_u64(&self.memory[(self.ip + 10) as usize..(self.ip + 18) as usize]);

        let abs_addr: u64 = self.data_base + rel_addr + offset;
        self.registers[r_dest_ind] = Register::uint(abs_addr);
        self.reg_types[r_dest_ind] = RegTypes::ds_addr;

        self.ip += 18;
        return;
    }
    fn op_dsderef(&mut self) {
        // 0x75, size: 11
        // dsderef Rsrc Rdest Offset
        let r_src_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_dest_ind: usize = self.memory[(self.ip + 2) as usize] as usize;
        let offset: usize =
            args_to_u64(&self.memory[(self.ip + 3) as usize..(self.ip + 11) as usize]) as usize;

        let src_val = self.registers[r_src_ind].as_u64() as usize;
        let val_type = self.memory[src_val - offset];
        if val_type == 0x4 {
            panic!(
                "CRITICAL: At Instruction {:#x}:\n String constant cannot be dereferenced. \nCoredump created.",
                self.ip
            );
        }

        let tgt_addr: usize = src_val - offset + 8 + 1; // 8 for length skip
        self.registers[r_dest_ind] =
            Register::uint(args_to_u64(&self.memory[tgt_addr..(tgt_addr + 8)]));
        self.reg_types[r_dest_ind] = match val_type {
            0x1 | 0x5 => RegTypes::uint64,
            0x2 | 0x6 => RegTypes::int64,
            0x3 | 0x7 => RegTypes::float64,
            0x4 => RegTypes::StrAddr, //wont be reached anyway
            other => {
                panic!("Unknown data type: {}", other);
            }
        };

        self.ip += 11;
        return;
    }
    fn op_dsrlea(&mut self) {
        // 0x76, size: 11
        // dsrlea Rdest Roffset Addr
        let r_dest_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_offset_ind: usize = self.memory[self.ip + 2] as usize;
        let rel_addr: u64 =
            args_to_u64(&self.memory[(self.ip + 3) as usize..(self.ip + 11) as usize]);
        let offset: u64 = self.registers[r_offset_ind].as_u64();

        let abs_addr: u64 = self.data_base + rel_addr + offset;
        self.registers[r_dest_ind] = Register::uint(abs_addr);
        self.reg_types[r_dest_ind] = RegTypes::ds_addr;

        self.ip += 11;
        return;
    }
    fn op_dsrderef(&mut self) {
        // 0x77, size: 4
        // dsrderef Rsrc Rdest Roffset
        let r_src_ind: usize = self.memory[(self.ip + 1) as usize] as usize;
        let r_dest_ind: usize = self.memory[(self.ip + 2) as usize] as usize;
        let r_offset_ind: usize = self.memory[(self.ip + 3) as usize] as usize;

        let offset: usize = self.registers[r_offset_ind].as_u64() as usize;

        let src_val = self.registers[r_src_ind].as_u64() as usize;
        let val_type = self.memory[src_val - offset];
        if val_type == 0x4 {
            if let Err(e) = self.err_coredump() {
                eprintln!("Error creating coredump: {}", e);
            };
            panic!(
                "CRITICAL: At Instruction {:#x}:\n String constant cannot be dereferenced. \nCoredump created.",
                self.ip
            );
        }

        let tgt_addr: usize = src_val - offset + 8 + 1; // 8 for length skip
        self.registers[r_dest_ind] =
            Register::uint(args_to_u64(&self.memory[tgt_addr..(tgt_addr + 8)]));
        self.reg_types[r_dest_ind] = match val_type {
            0x1 | 0x5 => RegTypes::uint64,
            0x2 | 0x6 => RegTypes::int64,
            0x3 | 0x7 => RegTypes::float64,
            0x4 => RegTypes::StrAddr, //wont be reached anyway
            other => {
                self.err_coredump();
                panic!(
                    "Unknown data type: {} at IP = {:#x}, src val at {:#x}",
                    other,
                    self.ip,
                    src_val - offset
                );
            }
        };

        self.ip += 4;
        return;
    }

    fn ncall_println(&mut self) {
        // size: 4
        let src_r_num: u8 = self.memory[self.ip + 3];
        match self.reg_types[src_r_num as usize] {
            RegTypes::uint64
            | RegTypes::int64
            | RegTypes::float64
            | RegTypes::address
            | RegTypes::ds_addr => {
                println!("{}", self.registers[src_r_num as usize]);
            }
            RegTypes::StrAddr => {
                let abs_addr: u64 = self.registers[src_r_num as usize].as_u64();
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
    pub fn coredump(&mut self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        let zeros: Vec<u8> = vec![0; 16];
        res.extend(&clone_placed(&self.memory));
        res.extend(&(zeros.clone()));

        // let stack_u8_cl: Vec<u8> = clone_placed_64(&self.stack)
        //     .iter()
        //     .flat_map(|num| num.to_be_bytes())
        //     .collect();
        // res.extend(&stack_u8_cl);
        res.extend(&zeros);
        res.extend(&(clone_placed(&self.heap.heap)));
        res
    }
    fn err_coredump(&mut self) -> std::result::Result<(), String> {
        let dump = self.coredump();
        let mut out_file: File = match File::create("voxvm_err.dump") {
            Ok(f) => f,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        if let Err(e) = out_file.write_all(&dump) {
            return Err(e.to_string());
        };
        Ok(())
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
    if s.is_empty() {
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

pub fn clone_placed(toclone: &Vec<u8>) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();
    for i in 0..toclone.len() {
        res.push(toclone[i].clone());
    }
    res
}

pub fn clone_placed_64(toclone: &Vec<u64>) -> Vec<u64> {
    let mut res: Vec<u64> = Vec::new();
    for i in 0..toclone.len() {
        res.push(toclone[i].clone());
    }
    res
}
