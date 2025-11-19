use std::{collections::HashMap, fs::{File, OpenOptions}};

use crate::{misclib::{bytes_into_string_utf16, show_runtime_err, u8_slice_to_u16_vec}, native::NSysError, vm::VM};

#[derive(Debug)]
pub enum FileModes {
    Write,
    Read,
    Append,
    ReadWrite,
    ReadAppend,
}

#[derive(Debug)]
pub struct NatSFile {
    pub file: File,
    pub mode: FileModes
}

impl NatSFile {
    pub fn new(f: File, m: FileModes) -> NatSFile {
        NatSFile { file: f, mode: m }
    }
}

#[derive(Debug)]
pub struct FileController {
    opened_files: Vec<NatSFile>,
}

impl FileController {
    pub fn new() -> FileController {
        FileController { 
            opened_files: (Vec::new()),
        }
    }

    pub fn open(&mut self, filename: String, mode: FileModes) 
        -> Result<usize, NSysError> {
        let mut options = OpenOptions::new();
        match mode {
            FileModes::Write => {
                options.write(true).create(true).truncate(true);
            },
            FileModes::Read => {
                options.read(true);
            },
            FileModes::Append => {
                options.write(true).create(true).append(true);
            },
            FileModes::ReadWrite => {
                options.write(true).read(true).create(true);
            }
            FileModes::ReadAppend => {
                options.read(true).write(true).append(true).create(true);
            }
        }
        let f = match options.open(filename) {
            Ok(v) => v,
            Err(e) => {
                return Err(NSysError::fs(e));
            }
        };
        let nf = NatSFile::new(f, mode);
        self.opened_files.push(nf);
        Ok(self.opened_files.len().saturating_sub(1))
    }
}

pub fn ncall_fopen(vm: &mut VM) {
    // r1 is heap ptr to filename string 
    // r2 is bytes count to read 
    // r3 is mode uint 
    // returns file index into r0 

    let from_ptr: u64 = vm.registers[1].as_u64();
    let count: u64 = vm.registers[2].as_u64();
    let mode_idx: u64 = vm.registers[3].as_u64();

    let fname_bytes: Vec<u8> = 
        match vm.heap.read(from_ptr, count) {
            Ok(b) => b,
            Err(()) => {
                show_runtime_err(vm, "Can't read heap!");
                return;
            }
    };
    let fname: String = String::from_utf16_lossy(
        &u8_slice_to_u16_vec(&fname_bytes)
    );

    let mode: FileModes = match mode_idx {
        1 => FileModes::Write,
        2 => FileModes::Read,
        3 => FileModes::Append,
        4 => FileModes::ReadWrite,
        5 => FileModes::ReadAppend,
        other => {
            show_runtime_err(vm, &format!("Unknown file mode: {}", mode_idx));
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    };

    vm.fc.open(fname, mode);
}

pub fn ncall_fclose(vm: &mut VM) {
    // ncall 0x11
    // r1 is file index 
    // returns result into r0 (0/1)
    let idx: usize = vm.registers[1].as_u64() as usize;
    if idx >= vm.fc.opened_files.len() {
        show_runtime_err(vm, "File index out of range");
        vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
        return;
    }   

    vm.fc.opened_files.remove(idx);
}

pub fn ncall_fwrite(vm: &mut VM) {
    // ncall 0x12 
    // r1 is file ind
    // r2 is heap ptr to start to copy
    // r3 is count 

    let f_idx: usize = vm.registers[1].as_u64() as usize;
    let tocopy: u64 = vm.registers[2].as_u64();
    let count: u64 = vm.registers[3].as_u64();

    if f_idx >= vm.fc.opened_files.len() {
        show_runtime_err(vm, "File index out of range");
        vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
        return;
    }    
    // finish ts 
}
