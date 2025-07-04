use std::io::{Seek, Write};
use std::{
    collections::hash_map::HashMap,
    fs::{self, File},
};

use crate::vm::args_to_u64;

#[derive(Debug)]
pub struct VoxExeHeader {
    // v3
    pub magic: [u8; 4],
    pub version: u16,
    pub entry_point: u64,
    pub data_base: u64,
    pub code_size: u64,
    pub data_size: u64,
    pub func_table_len: u64,  // number of funcs
    pub func_table: Vec<u64>, //Starts at 0x30
}

impl VoxExeHeader {
    pub fn new(
        ver: u16,
        entry: u64,
        data_start: u64,
        code_size: u64,
        data_size: u64,
        func_table: Vec<u64>,
    ) -> VoxExeHeader {
        let mag = b"VVE\0";
        VoxExeHeader {
            magic: *mag,
            version: ver,
            entry_point: entry,
            data_base: data_start,
            data_size: data_size,
            code_size: code_size,
            func_table_len: func_table.len() as u64,
            func_table: func_table,
        }
    }

    pub fn load(filename: &str, minVersion: u16) -> Result<VoxExeHeader, ()> {
        match fs::read(filename) {
            Ok(bytes) => {
                let magic = &bytes[0..4];
                if magic != b"VVE\0" {
                    eprintln!("Magic number of {} is incorrect.", filename);
                }

                let version: u16 = u16::from_be_bytes(bytes[4..6].try_into().unwrap());
                if version < minVersion {
                    panic!(
                        "{} file format version is {} and deprecated.",
                        filename, version
                    );
                }
                let entry_point: u64 = u64::from_be_bytes(bytes[6..14].try_into().unwrap());
                let data_base: u64 = u64::from_be_bytes(bytes[14..22].try_into().unwrap());
                let code_size: u64 = u64::from_be_bytes(bytes[22..30].try_into().unwrap());
                let data_size: u64 = u64::from_be_bytes(bytes[30..38].try_into().unwrap());
                let func_table_size: u64 = u64::from_be_bytes(bytes[38..46].try_into().unwrap());
                let func_table = Self::read_func_table(bytes.clone(), 0x30, func_table_size * 16);

                let magic_as_arr: [u8; 4] = magic[0..4].try_into().unwrap();

                Ok(VoxExeHeader {
                    magic: magic_as_arr,
                    version: version,
                    entry_point: entry_point,
                    data_base: data_base,
                    code_size: code_size,
                    data_size: data_size,
                    func_table_len: func_table_size,
                    func_table: func_table,
                })
            }
            Err(err) => {
                eprintln!(
                    "ERROR While reading .vve by path {}: \n
                {}",
                    filename, err
                );
                Err(())
            }
        }
    }

    pub fn read_func_table(file_bytes: Vec<u8>, start_ind: u64, count_bytes: u64) -> Vec<u64> {
        let mut res: Vec<u64> = vec![0; (count_bytes / 16) as usize];
        for i in (start_ind..start_ind + count_bytes).step_by(16) {
            let ind: u64 = args_to_u64(&file_bytes[(i as usize)..(i + 8) as usize]);
            let abs_addr: u64 = args_to_u64(&file_bytes[(i + 8) as usize..(i + 16) as usize]);
            res[ind as usize] = abs_addr;
        }
        res
    }

    pub fn write(filename: &str, header: &VoxExeHeader) -> File {
        let mut res: File = File::create(filename).unwrap();

        res.write_all(&header.magic);

        let vers = header.version.to_be_bytes();
        res.write_all(&vers);

        let entry = header.entry_point.to_be_bytes();
        res.write_all(&entry);

        let db = header.data_base.to_be_bytes();
        res.write_all(&db);

        let code_size = header.code_size.to_be_bytes();
        res.write_all(&code_size);

        let data_size = header.data_size.to_be_bytes();
        res.write_all(&data_size);

        let func_table_size = header.func_table_len.to_be_bytes();
        res.write_all(&func_table_size);

        let curpos = res.stream_position().unwrap();
        let tofill = (0x30 as usize).saturating_sub(curpos as usize);
        let zeros = vec![0; tofill];
        res.write_all(&zeros);

        //res.seek(std::io::SeekFrom::Start(0x30)); // func table starts from 0x30
        let func_table: Vec<u64> = header.func_table.clone();
        for (ind, addr) in func_table.iter().enumerate() {
            let ind_bytes = ind.to_be_bytes();
            let addr_bytes = addr.to_be_bytes();
            res.write_all(&ind_bytes);
            res.write_all(&addr_bytes);
        }

        res
    }

    pub fn write_existing(file: &mut File, header: &VoxExeHeader) {
        file.seek(std::io::SeekFrom::Start(0));
        file.write_all(&header.magic);

        let vers = header.version.to_be_bytes();
        file.write_all(&vers);

        let entry = header.entry_point.to_be_bytes();
        file.write_all(&entry);

        let db = header.data_base.to_be_bytes();
        file.write_all(&db);

        let code_size = header.code_size.to_be_bytes();
        file.write_all(&code_size);

        let data_size = header.data_size.to_be_bytes();
        file.write_all(&data_size);

        let func_table_size = header.func_table_len.to_be_bytes();
        file.write_all(&func_table_size);

        let curpos = file.stream_position().unwrap();
        let tofill = (0x30 as usize).saturating_sub(curpos as usize);
        let zeros = vec![0; tofill];
        file.write_all(&zeros);

        //file.seek(std::io::SeekFrom::Start(0x30)); // func table starts from 0x30
        let func_table: Vec<u64> = header.func_table.clone();
        for (ind, addr) in func_table.iter().enumerate() {
            let ind_bytes = ind.to_be_bytes();
            let addr_bytes = addr.to_be_bytes();
            file.write_all(&ind_bytes);
            file.write_all(&addr_bytes);
        }
    }
}
