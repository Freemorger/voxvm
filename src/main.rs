use std::{env, fs::File, io::Write, process::exit};

use assembly::VoxAssembly;
use vm::VM;

mod assembly;
mod fileformats;
mod vm;

fn main() {
    const DEFAULT_INIT_RAM: usize = 1024;
    const DEFAULT_INIT_STACK: usize = DEFAULT_INIT_RAM / 2;
    const DEFAULT_INIT_HEAP: usize = DEFAULT_INIT_RAM / 2;
    let mut ram_size: Option<usize> = None;
    let mut stack_size: Option<usize> = None;
    let mut heap_size: Option<usize> = None;

    let mut vvr_filename: Option<String> = None;
    let mut vve_filename: Option<String> = None;
    const MIN_VVE_VERSION: u16 = 2;

    let mut vas_input_filename: Option<String> = None;
    let mut vas_out_filename: Option<String> = None;

    let mut coredump_on_exit: bool = false;

    for arg in env::args() {
        if let Some(val) = arg.strip_prefix("--init-ram=") {
            match val.parse::<usize>() {
                Ok(num) => ram_size = Some(num),
                Err(_) => {
                    eprintln!("ERROR: Init ram value has to be integer.");
                    return;
                }
            }
        }
        if let Some(val) = arg.strip_prefix("--init-stack-size=") {
            match val.parse::<usize>() {
                Ok(num) => stack_size = Some(num),
                Err(_) => {
                    eprintln!("ERROR: Init stack size has to be integer.");
                    return;
                }
            }
        }
        if let Some(val) = arg.strip_prefix("--init-heap-size=") {
            match val.parse::<usize>() {
                Ok(num) => heap_size = Some(num),
                Err(_) => {
                    eprintln!("ERROR: Init heap size has to be integer.");
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
        if let Some(val) = arg.strip_prefix("--vas=") {
            match val.parse::<String>() {
                Ok(st) => vas_input_filename = Some(st.to_string()),
                Err(_) => {
                    eprintln!("ERROR: Parsing vas input filename error.");
                }
            }
        }
        if let Some(val) = arg.strip_prefix("--vas-out=") {
            match val.parse::<String>() {
                Ok(st) => vas_out_filename = Some(st.to_string()),
                Err(_) => {
                    eprintln!("ERROR: Parsing vas input filename error.");
                }
            }
        }
        if let Some(val) = arg.strip_prefix("--coredump_exit") {
            coredump_on_exit = true;
        }
    }

    match vas_input_filename {
        Some(st) => {
            let default_out_filename = st.replace(".vvs", ".vve");
            let mut asm =
                VoxAssembly::new(st, vas_out_filename.unwrap_or_else(|| default_out_filename));
            asm.assemble();
            return;
        }
        None => {}
    }

    match ram_size {
        Some(size) => println!("Initializing VM with init RAM size = {}", size),
        None => {
            println!(
                "Init RAM size is not specified, using {} bytes by default.",
                DEFAULT_INIT_RAM
            );
            ram_size = Some(DEFAULT_INIT_RAM);
        }
    }
    match stack_size {
        Some(size) => println!("Initializing VM with init stack size = {}", size),
        None => {
            println!(
                "Init stack size is not specified, using {} bytes by default.",
                DEFAULT_INIT_STACK
            );
            stack_size = Some(DEFAULT_INIT_STACK);
        }
    }
    match heap_size {
        Some(size) => println!("Initializing VM with init heap size = {}", size),
        None => {
            println!(
                "Init heap size is not specified, using {} bytes by default.",
                DEFAULT_INIT_HEAP
            );
            heap_size = Some(DEFAULT_INIT_HEAP);
        }
    }

    let mut vm_instance = VM::new(ram_size.unwrap(), stack_size.unwrap(), heap_size.unwrap());
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
    if (coredump_on_exit) {
        let dump = vm_instance.coredump();
        let mut out_file = match File::create("voxvm.dump") {
            Ok(f) => f,
            Err(e) => {
                println!("While saving coredump: {}", e.to_string());
                return;
            }
        };
        out_file.write_all(&dump);
    }
}
