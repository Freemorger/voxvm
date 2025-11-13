use rand::Rng;

use crate::{
    misclib::{bytes_into_string_utf16, show_runtime_err, string_from_straddr},
    registers::Register,
    vm::{RegTypes, VM},
};
use std::{char::decode_utf16, io::Write, process::{Command, Stdio}, thread::sleep, time::Duration};

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

pub fn readin(vm: &mut VM) {
    // r1 is rdst (heap pointer)
    // r2 is max to read 
    // returns red bytes count into r0
    let to_ptr = vm.registers[1].as_u64();
    let maxn: usize = vm.registers[2].as_u64() as usize;

    let mut input_st: String = String::new();
    match std::io::stdin().read_line(&mut input_st) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Runtime error: {}", e.to_string());
            vm.registers[0] = Register::uint(0);
            return;
        }
    }

    let dbytes: Vec<u16> = input_st
        .encode_utf16()
        .collect();
    let bytes: Vec<u8> = dbytes.iter()
        .flat_map(|db| db.to_be_bytes())
        .collect();
    let end: usize = maxn.clamp(1, 
        bytes.len().saturating_sub(1));
    
    match vm.heap.write(to_ptr, bytes[0..end].to_owned()) {
        Ok(()) => {},
        Err(()) => {
            eprintln!("Runtime error: Heap write");
            vm.registers[0] = Register::uint(0);
            vm.exceptions_active.push(crate::exceptions::Exception::HeapWriteFault);
            return;
        }
    }
    vm.registers[0] = Register::uint(end as u64);
}

pub fn randf(vm: &mut VM) {
    // returns random float in range 
    // 0..1 into r0 
   
    let val = rand::random::<f64>();
    vm.registers[0] = Register::float(val);
}

pub fn randint(vm: &mut VM) {
    // returns random signed int into r0
    // from range r1:r2 exclusively

    let low = vm.registers[1].as_i64();
    let high = vm.registers[2].as_i64();

    let val: i64 = vm.randgen.random_range(low..high);

    vm.registers[0] = Register::int(val);
}

pub fn getunixtime(vm: &mut VM) {
    // returns unix time as i64 into r0
    let time: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    vm.registers[0] = Register::int(time);
}

pub fn sleepcall(vm: &mut VM) {
    // r1 is u64 time in ms to sleep 
    let time: u64 = vm.registers[1].as_u64();

    sleep(Duration::from_millis(time));
}

pub fn runcmd(vm: &mut VM) {
    // r1 is heap ptr to command string bytes;
    // r2 is count bytes to read
    // r3 is heap ptr to place for saving stdout data
    // r4 is max bytes to write
    // returns count bytes of stdout into r0
    let ptr: u64 = vm.registers[1].as_u64();
    let count: u64 = vm.registers[2].as_u64();

    let bytes = match vm.heap.read(ptr, count) {
        Ok(b) => b,
        Err(()) => {
            show_runtime_err(vm, "Can't read heap");
            vm.exceptions_active.push(crate::exceptions::Exception::HeapReadFault);
            return;
        }
    };

    let st: String = match bytes_into_string_utf16(&bytes) {
        Some(v) => v,
        None => {
            show_runtime_err(vm, "Error converting bytes into string");
            vm.exceptions_active.push(crate::exceptions::Exception::HeapSegmFault);
            return;
        }
    };

    
    let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &st])
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(&st)
                .output()
                .expect("failed to execute process")
    };
    
    let out = output.stdout;
    let out_len = out.len();
    
    let out_ptr: u64 = vm.registers[3].as_u64();
    let maxc: usize = vm.registers[4]
        .as_u64()
        .clamp(0, out_len as u64) 
    as usize;

    match vm.heap.write(out_ptr, out[0..maxc].to_owned()) {
        Ok(()) => {},
        Err(()) => {
            show_runtime_err(vm, "Error writing stdout into heap");
            vm.exceptions_active.push(crate::exceptions::Exception::HeapWriteFault);
            return;
        }
    }

    vm.registers[0] = Register::uint(out_len as u64);
    
}
