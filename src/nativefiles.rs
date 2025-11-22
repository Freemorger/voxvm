use std::{collections::HashMap, fs::{File, OpenOptions}, io::{self, Read, Seek, Write}};

use crate::{misclib::{bytes_into_string_utf16, show_runtime_err, u8_slice_to_u16_vec}, native::NSysError, registers::Register, vm::VM};

#[derive(Debug, PartialEq)]
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
    pub mode: FileModes,
    pub path: String,
}

impl NatSFile {
    pub fn new(f: File, m: FileModes, path: String) -> NatSFile {
        NatSFile { file: f, mode: m, path: path }
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
        let mut f = match options.open(filename.clone()) {
            Ok(v) => v,
            Err(e) => {
                return Err(NSysError::fs(e));
            }
        };
        f.seek(io::SeekFrom::Start(0));
        let nf = NatSFile::new(f, mode, filename);
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

    let res = match vm.fc.open(fname, mode) {
        Ok(v) => v,
        Err(e) => {
            show_runtime_err(vm, &format!("FC error: {:#?}", e));
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    };

    vm.registers[0] = Register::uint(res as u64);
}

pub fn ncall_fclose(vm: &mut VM) {
    // ncall 0x11
    // r1 is file index 
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

    let mut f = match vm.fc.opened_files.get_mut(f_idx) {
        Some(v) => v,
        None => {
            show_runtime_err(vm, "File index out of range");
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    };

    if f.mode == FileModes::Read {
        show_runtime_err(vm, &format!("File with idx {} is readonly", f_idx));
        vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
        return;
    }

    let bytes = match vm.heap.read(tocopy, count) {
        Ok(v) => v,
        Err(()) => {
            show_runtime_err(vm, "Heap read fault!");
            vm.exceptions_active.push(crate::exceptions::Exception::HeapReadFault);
            return;
        }
    };

    if let Err(e) = f.file.write_all(&bytes) {
        show_runtime_err(vm, "Can't write buf into file!");
        vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
        return;
    }
    
    
}

pub fn ncall_fread(vm: &mut VM) {
    // ncall 0x12 
    // r1 is file idx 
    // r2 is bytes count 
    // r3 is heap dst ptr 
    // reads count bytes from file seek into vm heap 
    let f_idx = vm.registers[1].as_u64() as usize;
    let count = vm.registers[2].as_u64();
    let dst = vm.registers[3].as_u64();

    let mut f = match vm.fc.opened_files.get_mut(f_idx) {
        Some(v) => v,
        None => {
            show_runtime_err(vm, "File index out of range");
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    };

    if (f.mode == FileModes::Write) || (f.mode == FileModes::Append) {
        show_runtime_err(vm, &format!("File with idx {} is writeonly", f_idx));
        vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
        return;
    }

    let mut buf = vec![0u8; count as usize];
    let _ = f.file.read(&mut buf);

    if let Err(()) = vm.heap.write(dst, buf) {
        show_runtime_err(vm, "Can't write into heap!");
        vm.exceptions_active.push(crate::exceptions::Exception::HeapWriteFault);
        return;
    }
}

pub fn ncall_fdel(vm: &mut VM) {
    // ncall 0x14
    // r1 is file index 
    // deletes file from the filesystem AND filecontroller 
    let f_idx: usize = vm.registers[1].as_u64() as usize;

    if f_idx >= vm.fc.opened_files.len() {
        show_runtime_err(vm, "File index out of range");
        vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
        return;
};

    let f = vm.fc.opened_files.remove(f_idx);
    let fname = f.path.clone();

    drop(f);

    std::fs::remove_file(fname);
}

/// ncall 0x15
/// r1 is file index
/// will return current seek into r0
pub fn ncall_fseekget(vm: &mut VM) {
        let f_idx: usize = vm.registers[1].as_u64() as usize;
    
    let mut f = match vm.fc.opened_files.get_mut(f_idx) {
        Some(v) => v,
        None => {
            show_runtime_err(vm, "File index out of range");
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    };

    let seek: u64 = match f.file.stream_position() {
        Ok(v) => v,
        Err(e) => {
            show_runtime_err(vm, &format!("Error getting seek: {:#?}", e));
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return; 
        }
    };

    vm.registers[0] = Register::uint(seek);
}

/// ncall 0x16 
/// r1 is file index 
/// r2 is new seek (current one could be obtained from `ncall_fseekget`)
pub fn ncall_fseekset(vm: &mut VM) {
    let f_idx: usize = vm.registers[1].as_u64() as usize;
    let newseek: u64 = vm.registers[2].as_u64();

    let mut f = match vm.fc.opened_files.get_mut(f_idx) {
        Some(v) => v,
        None => {
            show_runtime_err(vm, "File index out of range");
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    };

    f.file.seek(io::SeekFrom::Start(newseek));
}
