use std::env;

use vm::VM;

mod tables;
mod vm;

fn main() {
    const DEFAULT_INIT_RAM: u64 = 1024;
    let mut ram_size: Option<u64> = None;

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
    let nvb_path = curdir.join("tools").join("program_asm.nvb");
    vm_instance.load_nvb(&nvb_path.to_string_lossy().to_string());
    vm_instance.run();
}
