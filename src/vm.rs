use std::{collections::HashMap, fmt::Result, fs};

#[derive(Debug, Clone, Copy)]
pub enum RegTypes {
    uint64,
    int64,
    float64,
    address,
}

#[derive(Debug)]
pub struct VM {
    registers: [u64; 32],
    reg_types: [RegTypes; 32],
    flags: [u8; 4], // of, zf, nf, cf
    ip: usize,
    memory: Vec<u8>, // dividing by each bytes, then can be grouped
    heap_ptr: usize,
    nativecalls: std::collections::HashMap<u16, NativeFn>,
    running: bool,
}
type NativeFn = fn(&mut VM, &[u64]) -> Result;
type InstructionHandler = fn(&mut VM);

impl VM {
    pub fn new(heap_size: usize) -> VM {
        VM {
            registers: [0; 32],
            reg_types: [RegTypes::uint64; 32],
            flags: [0; 4],
            ip: 0x0,
            memory: vec![0; heap_size],
            heap_ptr: 0,
            nativecalls: HashMap::new(),
            running: true,
        }
    }
    pub fn load_nvb(&mut self, input_file_name: &str) {
        let mut bctr: u64 = 0;
        match fs::read(input_file_name) {
            Ok(bytes) => {
                for byte in &bytes {
                    self.memory[bctr as usize] = *byte;
                    bctr += 1;
                }
            }
            Err(err) => panic!("CRITICAL: Can't read .nvb file. Error: {}", err),
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
        handlers[0x10] = Self::op_uload as InstructionHandler;
        handlers[0x11] = Self::op_uadd as InstructionHandler;
        handlers[0x12] = Self::op_umul as InstructionHandler;
        handlers[0x13] = Self::op_usub as InstructionHandler;
        handlers[0x14] = Self::op_udiv as InstructionHandler;
        handlers[0x15] = Self::op_urem as InstructionHandler;
        handlers[0x16] = Self::op_ucmp as InstructionHandler;
        handlers[0x20] = Self::op_iload as InstructionHandler;
        handlers[0x21] = Self::op_iadd as InstructionHandler;
        handlers[0x22] = Self::op_imul as InstructionHandler;
        handlers[0x40] = Self::op_jmp as InstructionHandler;
        handlers[0x41] = Self::op_jz as InstructionHandler;
        handlers[0x42] = Self::op_jl as InstructionHandler;
        handlers[0x43] = Self::op_jg as InstructionHandler;
        handlers[0x44] = Self::op_jge as InstructionHandler;
        handlers[0x45] = Self::op_jle as InstructionHandler;
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
        if (self.registers[(self.ip + 1)] == 0) {
            self.flags[1] = 1; // Zero flag
        } else {
            self.flags[1] = 0;
        }
        self.ip += 4;
    }

    fn op_urem(&mut self) {
        // 0x15, size: 4
        let reg_dest: u8 = self.memory[(self.ip + 1)];
        let reg_1: u8 = self.memory[(self.ip + 2)];
        let reg_2: u8 = self.memory[(self.ip + 3)];

        self.registers[reg_dest as usize] =
            self.registers[reg_1 as usize] % self.registers[reg_2 as usize];
        if (self.registers[reg_dest as usize] == 0) {
            self.flags[1] = 1; // Zero flag
        } else {
            self.flags[1] = 0;
        }
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

    fn op_jmp(&mut self) {
        // 0x40, size: 9
        let target_addr: u64 = args_to_u64(&self.memory[(self.ip + 1)..(self.ip + 9)]);
        self.ip = target_addr as usize;
        return;
    }

    fn op_jz(&mut self) {
        // 0x41, size: 9
        //println!("DBG: ZF = {}", self.flags[1]);
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
                println!("{}", f64::from_bits(self.registers[src_r_num as usize]));
            }
            RegTypes::address => {
                // TODO: Implement printing data from addr
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
