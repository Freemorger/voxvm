use core::panic;
use maplit::hashmap;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, Write},
};

use crate::fileformats::VoxExeHeader;
//use crate::fileformats::VoxExeHeader;

enum LexTypes {
    Op(u8),
    Size(u64), // size of instr in bytes
    NcallNum(u16),
    Reg(u8),
    Addr(u64),
    Value(u64),
}

pub struct VoxAssembly {
    cur_addr: u64,
    entry: u64,
    data_start: u64,
    labels: HashMap<String, u64>,
    instr_table: HashMap<String, Vec<LexTypes>>,
    bin_buffer: Vec<u8>,
    input_file: File,
    output_file: File,
    read_buffer: BufReader<File>,
    is_vve: bool,
}

impl VoxAssembly {
    pub fn new(input_filename: String, output_filename: String) -> VoxAssembly {
        let is_vve: bool = match (output_filename.contains(".vve")) {
            true => true,
            false => false,
        };
        let default_entry: u64 = 0;
        let mut labels: HashMap<String, u64> = HashMap::new();
        let mut buf: Vec<u8> = Vec::new();

        let mut in_file: File;
        {
            let _out = match File::create(output_filename.clone()) {
                Ok(file) => file,
                Err(err) => panic!(
                    "ERROR: ERROR: While creating output file for voxasm: {}",
                    err
                ),
            };
        }
        let mut out_file: File = OpenOptions::new()
            .append(true)
            .open(output_filename)
            .unwrap();

        match File::open(input_filename) {
            Ok(file) => in_file = file,
            Err(err) => panic!("ERROR: While opening input voxasm file: {}", err),
        }

        VoxAssembly {
            cur_addr: 0x0,
            entry: default_entry,
            data_start: 0x0,
            labels: labels,
            instr_table: voxasm_instr_table(),
            bin_buffer: buf,
            output_file: out_file,
            read_buffer: BufReader::new(in_file.try_clone().unwrap()),
            input_file: in_file,
            is_vve: is_vve,
        }
    }

    pub fn assemble(&mut self) {
        self.first_stage();
        self.cur_addr = 0;
        self.read_buffer.seek(std::io::SeekFrom::Start((0)));

        let lines: Vec<_> = self.read_buffer.by_ref().lines().collect();
        for line in lines {
            let line = line.unwrap();
            let lexems: Vec<&str> = line.trim().split_whitespace().collect();
            if lexems.is_empty() {
                continue;
            }
            //println!("DBG Lexems: {}", lexems.join(", "));
            if (lexems[0] == "label")
                || (lexems[0] == ".start")
                || (lexems[0] == ".data")
                || (lexems[0].contains("#") || (lexems[0] == ";"))
            {
                continue;
            }

            let instr_data = match self.instr_table.get(lexems[0]) {
                Some(val) => val,
                None => {
                    eprintln!("ERR: No such instruction '{}'", lexems[0]);
                    continue;
                }
            };
            let opcode = match &instr_data[0] {
                LexTypes::Op(value) => *value,
                _ => panic!("ERR: First element should be an Op variant"),
            };
            let instr_len = match &instr_data[1] {
                LexTypes::Size(value) => *value,
                _ => panic!("ERR: Second element should be an Size variant"),
            };
            self.bin_buffer.push(opcode as u8);

            if (opcode >= 0x40) && (opcode <= 0x50) {
                let get_addr = self.labels.get(lexems[1]);
                let mut tgt_addr: u64 = 0;
                match get_addr {
                    Some(addr) => tgt_addr = *addr,
                    None => tgt_addr = lexems[1].parse().unwrap(),
                }
                //println!("DBG Target addr: {}", tgt_addr);
                self.bin_buffer.extend_from_slice(&tgt_addr.to_be_bytes());
                continue;
            }
            for arg in &lexems[1..] {
                if (arg.contains("#") || (arg == &";")) {
                    break;
                }

                if (arg.contains("r")) {
                    let reg_ind: u8 = arg[1..].parse().unwrap();
                    self.bin_buffer.push(reg_ind);
                    continue;
                }
                if (arg.contains(".")) {
                    let val: f64 = arg.parse().unwrap();
                    let res = val.to_be_bytes();
                    self.bin_buffer.extend_from_slice(&res);
                    continue;
                }

                let mut is_signed: bool = false;
                if (opcode >= 0x20) && (opcode <= 0x30) {
                    is_signed = true;
                }

                let mut res: [u8; 8];
                let mut signed_res: i64;
                let mut unsigned_res: u64;
                let mut num_sys: u32 = 10;
                let mut bytes_limit: usize = 8;

                if (opcode == 0x1) {
                    bytes_limit = 2;
                }
                if (arg.to_lowercase().contains("0x")) {
                    num_sys = 16;
                }

                if (is_signed) {
                    signed_res = i64::from_str_radix(arg, num_sys).unwrap();
                    res = signed_res.to_be_bytes();
                } else {
                    unsigned_res = u64::from_str_radix(arg, num_sys).unwrap();
                    res = unsigned_res.to_be_bytes();
                }
                self.bin_buffer
                    .extend_from_slice(&res[res.len() - bytes_limit..]);
            }
        }
        if (self.is_vve) {
            self.do_vve();
        } else {
            self.do_vvr();
        }
    }

    fn save_label(&mut self, labelname: String) {
        let addr = self.entry + self.cur_addr;
        self.labels.insert(labelname, addr);
        return;
    }

    fn first_stage(&mut self) {
        let lines: Vec<_> = self.read_buffer.by_ref().lines().collect();
        for line in lines {
            let line = line.unwrap();
            let lexems: Vec<&str> = line.trim().split_whitespace().collect();
            if lexems.is_empty() {
                continue;
            }

            if (lexems[0] == "label") {
                self.save_label(lexems[1].to_string());
                continue;
            } else if (lexems[0] == "goto") {
                self.cur_addr += 9;
                continue;
            } else if (lexems[0] == ".start") {
                self.entry = self.cur_addr;
                continue;
            } else if (lexems[0].contains("#") || lexems[0] == ";") {
                continue;
            } else {
                let instr_data = self.instr_table.get(lexems[0]).unwrap();
                let instr_size = match instr_data[1] {
                    LexTypes::Size(val) => val,
                    _ => {
                        eprintln!("Error parsing inside label parse: can't fetch instr_size");
                        0
                    }
                };
                self.cur_addr += instr_size;
            }
        }
    }

    fn do_vvr(&mut self) {
        match self.output_file.write_all(&self.bin_buffer) {
            Ok(_) => return,
            Err(err) => panic!("ERR: While writing bytecode into output .vvr file: {}", err),
        }
    }

    fn do_vve(&mut self) {
        const VVE_VERSION: u16 = 2;
        let header: VoxExeHeader = VoxExeHeader::new(
            VVE_VERSION,
            self.entry,
            self.data_start,
            0, // this fields currently unudsed
            0,
        );
        VoxExeHeader::write_existing(&mut self.output_file, &header);
        match self.output_file.write_all(&self.bin_buffer) {
            Ok(_) => return,
            Err(err) => panic!("ERR: While writing bytecode into output .vve file: {}", err),
        }
    }
}

fn voxasm_instr_table() -> HashMap<String, Vec<LexTypes>> {
    // Format:
    // Opcode, length, args.
    hashmap! {
        "halt".to_string() => vec![LexTypes::Op(0xFF), LexTypes::Size(1)],
        "ncall".to_string() => vec![LexTypes::Op(0x1), LexTypes::Size(4), LexTypes::NcallNum(0), LexTypes::Reg(0)],
        "nop".to_string() => vec![LexTypes::Op(0x2), LexTypes::Size(1)],
        "uload".to_string() => vec![LexTypes::Op(0x10), LexTypes::Size(10), LexTypes::Reg(0), LexTypes::Value(0)],
        "uadd".to_string() => vec![LexTypes::Op(0x11), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "umul".to_string() => vec![LexTypes::Op(0x12), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "usub".to_string() => vec![LexTypes::Op(0x13), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "udiv".to_string() => vec![LexTypes::Op(0x14), LexTypes::Size(4), LexTypes::Reg(0), LexTypes::Reg(0), LexTypes::Reg(0)],
        "urem".to_string() => vec![LexTypes::Op(0x15), LexTypes::Size(4), LexTypes::Reg(0), LexTypes::Reg(0), LexTypes::Reg(0)],
        "ucmp".to_string() => vec![LexTypes::Op(0x16), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "usqrt".to_string() => vec![LexTypes::Op(0x17), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "upow".to_string() => vec![LexTypes::Op(0x18), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "iload".to_string() => vec![LexTypes::Op(0x20), LexTypes::Size(10), LexTypes::Reg(0), LexTypes::Value(0)],
        "iadd".to_string() => vec![LexTypes::Op(0x21), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "imul".to_string() => vec![LexTypes::Op(0x22), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "isub".to_string() => vec![LexTypes::Op(0x23), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "idiv".to_string() => vec![LexTypes::Op(0x24), LexTypes::Size(4), LexTypes::Reg(0), LexTypes::Reg(0), LexTypes::Reg(0)],
        "irem".to_string() => vec![LexTypes::Op(0x25), LexTypes::Size(4), LexTypes::Reg(0), LexTypes::Reg(0), LexTypes::Reg(0)],
        "icmp".to_string() => vec![LexTypes::Op(0x26), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "iabs".to_string() => vec![LexTypes::Op(0x27), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "ineg".to_string() => vec![LexTypes::Op(0x28), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "isqrt".to_string() => vec![LexTypes::Op(0x29), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "ipow".to_string() => vec![LexTypes::Op(0x2a), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "fload".to_string() => vec![LexTypes::Op(0x30), LexTypes::Size(10), LexTypes::Reg(0), LexTypes::Value(0)],
        "fadd".to_string() => vec![LexTypes::Op(0x31), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "fmul".to_string() => vec![LexTypes::Op(0x32), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "fsub".to_string() => vec![LexTypes::Op(0x33), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "fdiv".to_string() => vec![LexTypes::Op(0x34), LexTypes::Size(4), LexTypes::Reg(0), LexTypes::Reg(0), LexTypes::Reg(0)],
        "frem".to_string() => vec![LexTypes::Op(0x35), LexTypes::Size(4), LexTypes::Reg(0), LexTypes::Reg(0), LexTypes::Reg(0)],
        "fcmp".to_string() => vec![LexTypes::Op(0x36), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "fcmp_eps".to_string() => vec![LexTypes::Op(0x37), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "fabs".to_string() => vec![LexTypes::Op(0x38), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "fneg".to_string() => vec![LexTypes::Op(0x39), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "fsqrt".to_string() => vec![LexTypes::Op(0x3a), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "fpow".to_string() => vec![LexTypes::Op(0x3b), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "jmp".to_string() => vec![LexTypes::Op(0x40), LexTypes::Size(9), LexTypes::Addr(0)],
        "jz".to_string() => vec![LexTypes::Op(0x41), LexTypes::Size(9), LexTypes::Addr(0)],
        "jl".to_string() => vec![LexTypes::Op(0x42), LexTypes::Size(9), LexTypes::Addr(0)],
        "jg".to_string() => vec![LexTypes::Op(0x43), LexTypes::Size(9), LexTypes::Addr(0)],
        "jge".to_string() => vec![LexTypes::Op(0x44), LexTypes::Size(9), LexTypes::Addr(0)],
        "jle".to_string() => vec![LexTypes::Op(0x45), LexTypes::Size(9), LexTypes::Addr(0)],
        "utoi".to_string() => vec![LexTypes::Op(0x50), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "itou".to_string() => vec![LexTypes::Op(0x51), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "utof".to_string() => vec![LexTypes::Op(0x52), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "itof".to_string() => vec![LexTypes::Op(0x53), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "ftou".to_string() => vec![LexTypes::Op(0x54), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "ftoi".to_string() => vec![LexTypes::Op(0x55), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "movr".to_string() => vec![LexTypes::Op(0x60), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "or".to_string() => vec![LexTypes::Op(0x61), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "and".to_string() => vec![LexTypes::Op(0x62), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "not".to_string() => vec![LexTypes::Op(0x63), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "xor".to_string() => vec![LexTypes::Op(0x64), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "test".to_string() => vec![LexTypes::Op(0x65), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)],
        "lnot".to_string() => vec![LexTypes::Op(0x66), LexTypes::Size(3), LexTypes::Reg(0), LexTypes::Reg(0)]
    }
}
