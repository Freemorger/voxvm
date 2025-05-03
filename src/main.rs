use std::{env, process::exit};

use vm::VM;

mod fileformats;
mod vm;

fn main() {
    const DEFAULT_INIT_RAM: u64 = 1024;
    let mut ram_size: Option<u64> = None;
    let mut vvr_filename: Option<String> = None;
    let mut vve_filename: Option<String> = None;
    const MIN_VVE_VERSION: u16 = 1;

    for arg in env::args() {
        if let Some(val) = arg.strip_prefix("--init-ram=") {
            match val.parse::<u64>() {
                Ok(num) => ram_size = Some(num),
                Err(_) => {
                    eprintln!("ERROR: Init ram value has to be integer.");
                    return;
                }
            }
        }
        if let Some(val) = arg.strip_prefix("--vvr=") {
            match val.parse::<String>() {
                Ok(st) => vvr_filename = Some(st.to_string()),
                Err(_) => {
                    eprintln!("ERROR: Parsing filename error.");
                }
            }
        }
        if let Some(val) = arg.strip_prefix("--vve=") {
            match val.parse::<String>() {
                Ok(st) => vve_filename = Some(st.to_string()),
                Err(_) => {
                    eprintln!("ERROR: Parsing filename error.");
                }
            }
        }
    }

    match ram_size {
        Some(size) => println!("Initializing VM with init RAM size = {}", size),
        None => {
            println!("Init RAM size is not specified, using 1024 bytes by default.");
            ram_size = Some(DEFAULT_INIT_RAM);
        }
    }

    let mut vm_instance = VM::new(ram_size.unwrap() as usize);
    let curdir = env::current_dir().unwrap();

    match vvr_filename {
        Some(ref st) => vm_instance.load_vvr(&st),
        None => {}
    }
    match vve_filename {
        Some(ref st) => vm_instance.load_vve(&st, MIN_VVE_VERSION),
        None => {}
    }
    if (vvr_filename.is_none()) && (vve_filename.is_none()) {
        println!("Usage: voxvm --vvr=name - loads a vvr (voxvm raw) file;");
        println!("\t --vve=name - loads a vve (voxvm executable) file.");
        exit(0);
    }

    vm_instance.run();
}
