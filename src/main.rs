use std::{env, fs::File, io::Write, process::exit};

use assembly::VoxAssembly;
use regex::Regex;
use sysinfo::System;
use vm::VM;

mod assembly;
mod callstack;
mod exceptions;
mod fileformats;
mod func_ops;
mod gc;
mod heap;
mod native;
mod stack;
mod vm;

fn main() {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let available_ram = sys.available_memory();
    let sysram_multiplier: f64 = 0.001f64;

    let DEFAULT_INIT_RAM: usize = (available_ram as f64 * sysram_multiplier).round() as usize;
    let DEFAULT_INIT_STACK: usize = DEFAULT_INIT_RAM / 2;
    let DEFAULT_INIT_HEAP: usize = DEFAULT_INIT_RAM / 2;
    const DEFAULT_RECURSION_LIMIT: usize = 1000;
    let mut ram_size: Option<usize> = None;
    let mut stack_size: Option<usize> = None;
    let mut heap_size: Option<usize> = None;

    let mut vvr_filename: Option<String> = None;
    let mut vve_filename: Option<String> = None;
    const MIN_VVE_VERSION: u16 = 3;

    let mut vas_input_filename: Option<String> = None;
    let mut vas_out_filename: Option<String> = None;

    let mut coredump_on_exit: bool = false;

    let mut recursion_depth_limit: Option<usize> = None;

    let mut native_cfgs: Option<String> = None;

    for arg in env::args() {
        if let Some(val) = arg.strip_prefix("--init-ram=") {
            match pretty_input_tobytes(val.to_string()) {
                Some(num) => ram_size = Some(num),
                None => {
                    eprintln!(
                        "ERROR: Init ram value is incorrect.\nHint: specify unit, e.g. `--init-ram=100MB`"
                    );
                    return;
                }
            }
        }
        if let Some(val) = arg.strip_prefix("--init-stack-size=") {
            match pretty_input_tobytes(val.to_string()) {
                Some(num) => stack_size = Some(num),
                None => {
                    eprintln!(
                        "ERROR: Init stack size is incorrect.\nHint: specify unit, e.g. `--init-stack-size=100MB`"
                    );
                    return;
                }
            }
        }
        if let Some(val) = arg.strip_prefix("--init-heap-size=") {
            match pretty_input_tobytes(val.to_string()) {
                Some(num) => heap_size = Some(num),
                None => {
                    eprintln!(
                        "ERROR: Init heap size is incorrect.\nHint: specify unit, e.g. `--init-heap-size=100MB`"
                    );
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
        if let Some(val) = arg.strip_prefix("--max-recursion=") {
            match val.parse::<usize>() {
                Ok(v) => {
                    recursion_depth_limit = Some(v);
                }
                Err(_) => {}
            }
        }
        if let Some(val) = arg.strip_prefix("--native-configs=") {
            match val.parse::<String>() {
                Ok(st) => native_cfgs = Some(st.to_string()),
                Err(_) => {
                    eprintln!("ERROR: Parsing native-configs error.");
                }
            }
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
        Some(size) => println!(
            "Initializing VM with init RAM size = {}",
            pretty_fmt_size(size as u64)
        ),
        None => {
            println!(
                "Init RAM size is not specified, using {} by default.",
                pretty_fmt_size(DEFAULT_INIT_RAM as u64)
            );
            ram_size = Some(DEFAULT_INIT_RAM);
        }
    }
    match stack_size {
        Some(size) => println!(
            "Initializing VM with init stack size = {}",
            pretty_fmt_size(size as u64)
        ),
        None => {
            println!(
                "Init stack size is not specified, using {} by default.",
                pretty_fmt_size(DEFAULT_INIT_STACK as u64)
            );
            stack_size = Some(DEFAULT_INIT_STACK);
        }
    }
    match heap_size {
        Some(size) => println!(
            "Initializing VM with init heap size = {}",
            pretty_fmt_size(size as u64)
        ),
        None => {
            println!(
                "Init heap size is not specified, using {} by default.",
                pretty_fmt_size(DEFAULT_INIT_HEAP as u64)
            );
            heap_size = Some(DEFAULT_INIT_HEAP);
        }
    }

    let mut vm_instance = VM::new(
        ram_size.unwrap(),
        stack_size.unwrap(),
        heap_size.unwrap(),
        recursion_depth_limit.unwrap_or(DEFAULT_RECURSION_LIMIT),
    );
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
        println!("\t More info/args in README.md");
        exit(0);
    }

    match native_cfgs {
        Some(v) => {
            let res = vm_instance.nativesys.read_cfg(&v);
            match res {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("ERROR While parsing native conf: {}", e.to_string());
                }
            }
        }
        None => {}
    }

    vm_instance.run();
    if coredump_on_exit {
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

pub fn pretty_input_tobytes(s: String) -> Option<usize> {
    let re = Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(b|kb|mb|gb)").unwrap();

    for cap in re.captures_iter(&s) {
        let size = &cap[1];
        let unit = &cap[2];

        let multiplier: u64 = match unit.to_lowercase().as_str() {
            "gb" => 1024 * 1024 * 1024, // why not pow? because.
            "mb" => 1024 * 1024,
            "kb" => 1024,
            "b" => 1,
            _ => 0, // we wont reach it
        };
        let size_u64: f64 = size.parse::<f64>().unwrap();
        let res: usize = (size_u64 * (multiplier as f64)).round() as usize;
        return Some(res);
    }
    None
}

pub fn pretty_fmt_size(size: u64) -> String {
    if size >= (1024 * 1024 * 1024) {
        let gbytes: f64 = size as f64 / (1024 * 1024 * 1024) as f64;
        return format!("{:.1}GB", gbytes);
    }
    if size >= (1024 * 1024) {
        let mbytes: f64 = size as f64 / (1024 * 1024) as f64;
        return format!("{:.1}MB", mbytes);
    }
    if size >= (1024) {
        let kbytes: f64 = size as f64 / 1024 as f64;
        return format!("{:.1}KB", kbytes);
    }
    return format!("{}B", size);
}
