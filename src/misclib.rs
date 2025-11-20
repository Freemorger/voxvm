use crate::{
    native::VMValue,
    registers::Register,
    vm::{RegTypes, RegistersCount, VM},
};

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

pub fn pad_to(bytes: Vec<u8>, tgt_size: usize) -> Vec<u8> {
    let mut res = bytes;
    while res.len() < tgt_size {
        res.insert(0, 0);
    }
    res
}

pub fn u8_slice_to_u16_vec(bytes: &[u8]) -> Vec<u16> {
    bytes
        .chunks(2)
        .filter_map(|chunk| {
            if chunk.len() == 2 {
                Some(u16::from_be_bytes([chunk[0], chunk[1]]))
            } else {
                None
            }
        })
        .collect()
}

pub fn clone_placed(toclone: &Vec<u8>) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();
    for i in 0..toclone.len() {
        res.push(toclone[i].clone());
    }
    res
}

pub fn reg_into_vmval(reg: Register) -> VMValue {
    match reg {
        Register::uint(v) => VMValue {
            typeind: RegTypes::uint64 as u32,
            data: v,
        },
        Register::int(v) => VMValue {
            typeind: RegTypes::int64 as u32,
            data: reg.as_u64(),
        },
        Register::float(v) => VMValue {
            typeind: RegTypes::float64 as u32,
            data: reg.as_u64(),
        },
        Register::StrAddr(v) => VMValue {
            typeind: RegTypes::StrAddr as u32,
            data: reg.as_u64(),
        },
        Register::address(v) => VMValue {
            typeind: RegTypes::address as u32,
            data: reg.as_u64(),
        },
        Register::ds_addr(v) => VMValue {
            typeind: RegTypes::ds_addr as u32,
            data: reg.as_u64(),
        },
    }
}

// rust's TryFrom is dumb
pub fn RegTFromU32(u: u32) -> Option<RegTypes> {
    match u {
        1 => Some(RegTypes::uint64),
        2 => Some(RegTypes::int64),
        3 => Some(RegTypes::float64),
        4 => Some(RegTypes::StrAddr),
        8 => Some(RegTypes::address),
        9 => Some(RegTypes::ds_addr),
        _ => None,
    }
}

pub fn CollectRegsVMVal(regs: &[Register]) -> [VMValue; RegistersCount] {
    let mut res = [VMValue {
        data: 0,
        typeind: 0,
    }; RegistersCount];
    for (i, v) in regs.iter().enumerate() {
        res[i] = reg_into_vmval(*v);
    }
    res
}

pub fn string_from_straddr(vm: &mut VM, abs_addr: u64) -> Option<String> {
    let bytes_len = &vm.memory[((abs_addr - 8) as usize)..((abs_addr) as usize)];
    let size: u64 = u64::from_be_bytes(bytes_len.try_into().unwrap());

    let bytes_str = &vm.memory[(abs_addr as usize)..((abs_addr + size) as usize)];
    bytes_into_string_utf16(bytes_str)
}

pub fn bytes_into_string_utf16(bytes: &[u8]) -> Option<String> {
    let utf16_data = u8_slice_to_u16_vec(bytes);

    let res_str: String = match String::from_utf16(&utf16_data) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: While converting into printable string: {}", err);
            return None;
        }
    };
    Some(res_str)
}

/// Pretty prints runtime error
pub fn show_runtime_err(vm: &mut VM, msg: &str) {
    eprintln!("Runtime error occured! 
        \nAt IP = {:#x} (instr {:#x}):
        \n\t{}", vm.ip, vm.memory[vm.ip], msg);
}

pub fn vec16_into_vec8(v: Vec<u16>) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();
    for db in v {
        res.extend_from_slice(&db.to_be_bytes());
    }
    res
}
