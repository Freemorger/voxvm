use crate::{
    misclib::{bytes_into_string_utf16, string_from_straddr},
    registers::Register,
    vm::{RegTypes, VM},
};
use std::{char::decode_utf16, io::Write};

pub fn ncall_print(vm: &mut VM) {
    // r1 is rsrc (any type), r2 is stream id (1 for stdout, 2 for stderr),
    // r3 is count bytes to print, if heap addr
    let rsrc: Register = vm.registers[1];
    let stream_id: u64 = vm.registers[2].as_u64_bitwise();

    match rsrc {
        Register::uint(v) => {
            let st: String = v.to_string();
            print_stream(stream_id, st);
        }
        Register::int(v) => {
            let st: String = v.to_string();
            print_stream(stream_id, st);
        }
        Register::float(v) => {
            let st: String = v.to_string();
            print_stream(stream_id, st);
        }
        Register::StrAddr(v) => {
            let st: String = match string_from_straddr(vm, v) {
                Some(v) => v,
                None => {
                    eprintln!("ERROR: no res string!");
                    return;
                }
            };
            print_stream(stream_id, st);
        }
        Register::ds_addr(v) => {
            print_stream(stream_id, format!("VM Data segment address: 0x{:x}", v));
        }
        Register::address(v) => {
            let count: u64 = vm.registers[3].as_u64();
            if (vm.reg_types[3] == RegTypes::uint64) && (count > 0) {
                let bytes = match vm.heap.read(v, count) {
                    Ok(bv) => match bytes_into_string_utf16(&bv) {
                        Some(s) => {
                            print_stream(stream_id, s);
                            return;
                        }
                        None => {}
                    },
                    Err(_) => {
                        eprintln!("Failed to read bytes [0x{:x}]:[0x{:x}]", v, v + count);
                    }
                };
            }
            print_stream(stream_id, format!("VM Heap address: 0x{:x}", v));
        }
    }
}

fn print_stream(stream_id: u64, val: String) -> Result<(), ()> {
    match stream_id {
        1 => {
            // stdout
            println!("{}", val);
            std::io::stdout().flush();
        }
        2 => {
            // stderr
            eprintln!("{}", val);
            std::io::stderr().flush();
        }
        other => {
            return Err(());
        }
    }
    Ok(())
}
