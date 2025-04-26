use std::{collections::HashMap, fmt::Result, fs};

#[derive(Debug)]
pub struct VM {
    registers: [u64; 32],
    flags: [u8; 3], // of, zf, nf
    ip: usize,
    memory: Vec<u8>, // dividing by each bytes, then can be grouped
    heap_ptr: usize,
    nativecalls: std::collections::HashMap<u32, NativeFn>,
}
type NativeFn = fn(&mut VM, &[u64]) -> Result;

impl VM {
    pub fn new(heap_size: usize) -> VM {
        VM {
            registers: [0; 32],
            flags: [0; 3],
            ip: 0x0,
            memory: vec![0; heap_size],
            heap_ptr: 0,
            nativecalls: HashMap::new(),
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
        while (self.ip < self.memory.len()) {
            let opcode = self.memory[self.ip];
            match opcode {
                0x0 => break, // halt
                0x1 => self.op_ncall(),
                0x10 => self.op_uload(),
                0x11 => self.op_uadd(),
                // 20s for signed values
                // 30s for floats
                0x40 => self.op_jmp(),
                _ => panic!("Unknown operation code: {}", opcode),
            }
        }
    }

    fn op_ncall(&mut self) {
        let ncall_num: u8 = self.memory[(self.ip + 1) as usize];
        match ncall_num {
            0x1 => self.ncall_println(),
            _ => panic!("Unknown ncall code: {}", ncall_num),
        }
    }

    fn op_uload(&mut self) {
        let register_ind: u8 = self.memory[(self.ip + 1) as usize];
        let value: u64 = args_to_u64(&self.memory[(self.ip + 2)..(self.ip + 10)]);

        self.registers[register_ind as usize] = value;
        self.ip += 10;
        return;
    }

    fn op_uadd(&mut self) {
        let in_reg_ind: u8 = self.memory[(self.ip + 1) as usize];
        let toadd_reg_ind: u8 = self.memory[(self.ip + 2) as usize];

        self.registers[in_reg_ind as usize] += self.registers[toadd_reg_ind as usize];
        self.ip += 3;
        return;
    }

    fn op_jmp(&mut self) {
        let target_addr: u64 = args_to_u64(&self.memory[(self.ip + 1)..(self.ip + 9)]);
        self.ip = target_addr as usize;
        return;
    }

    fn ncall_println(&mut self) {
        let src_r_num: u8 = self.memory[(self.ip + 2)];
        println!("{}", self.registers[src_r_num as usize]);
        self.ip += 3;
        return;
    }
}

pub fn args_to_u64(args: &[u8]) -> u64 {
    let bytes: [u8; 8] = args.try_into().expect(&format!("Bytes convertion error!"));
    let value: u64 = u64::from_be_bytes(bytes);
    value
}
