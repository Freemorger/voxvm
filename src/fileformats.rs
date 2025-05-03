use std::fs::{self, File};
use std::io::Write;

#[derive(Debug)]
pub struct VoxExeHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub entry_point: u64,
    pub data_ptr: u64,
    pub code_size: u64,
}

impl VoxExeHeader {
    pub fn load(filename: &str, minVersion: u16) -> Result<VoxExeHeader, ()> {
        match fs::read(filename) {
            Ok(bytes) => {
                let magic = &bytes[0..4];
                if magic != b"VVE\0" {
                    eprintln!("Magic number of {} is incorrect.", filename);
                }

                let version: u16 = u16::from_be_bytes(bytes[4..6].try_into().unwrap());
                if (version < minVersion) {
                    eprintln!(
                        "{} file format version is {} and deprecated.",
                        filename, version
                    );
                }
                let entry_point: u64 = u64::from_be_bytes(bytes[6..14].try_into().unwrap());
                let data_ptr: u64 = u64::from_be_bytes(bytes[14..22].try_into().unwrap());
                let code_size: u64 = u64::from_be_bytes(bytes[22..30].try_into().unwrap());

                let magic_as_arr: [u8; 4] = magic[0..4].try_into().unwrap();

                Ok(VoxExeHeader {
                    magic: magic_as_arr,
                    version: version,
                    entry_point: entry_point,
                    data_ptr: data_ptr,
                    code_size: code_size,
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

    pub fn write(filename: &str, header: &VoxExeHeader) -> File {
        let mut res: File = File::create(filename).unwrap();

        res.write_all(&header.magic);

        let vers = header.version.to_be_bytes();
        res.write_all(&vers);

        let entry = header.entry_point.to_be_bytes();
        res.write_all(&entry);

        let dp = header.data_ptr.to_be_bytes();
        res.write_all(&dp);

        let code_size = header.code_size.to_be_bytes();
        res.write_all(&code_size);

        res
    }
}
