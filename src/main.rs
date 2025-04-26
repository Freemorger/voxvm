use std::env;

use vm::VM;

mod tables;
mod vm;

fn main() {
    let mut vm_instance = VM::new(1024);
    let curdir = env::current_dir().unwrap();
    let nvb_path = curdir.join("tools").join("program.nvb");
    vm_instance.load_nvb(&nvb_path.to_string_lossy().to_string());
    vm_instance.run();
}
