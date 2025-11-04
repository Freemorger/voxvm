use std::{
    collections::HashMap,
    env,
    ffi::c_void,
    fs::{File, read_dir},
    io,
};

use libloading::{Library, Symbol};
use serde::Deserialize;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VMValue {
    pub typeind: u32,
    pub data: u64,
}
type VMFFIFunction = unsafe extern "C" fn(args: *const VMValue, len: u32) -> VMValue;

#[derive(Debug)]
pub struct NativeService {
    libs: Vec<NativeLibrary>,
    platform: NSysOS,
    ncall_codes: HashMap<u16, (usize, NFuncCfg)>, // value is (lib ind, funcname)
}

impl NativeService {
    pub fn new() -> NativeService {
        #[cfg(target_os = "windows")]
        let os = NSysOS::Windows;

        #[cfg(target_os = "linux")]
        let os = NSysOS::Linux;

        #[cfg(target_os = "macos")]
        let os = NSysOS::MacOS;

        NativeService {
            libs: (Vec::new()),
            platform: os,
            ncall_codes: HashMap::new(),
        }
    }

    pub fn read_cfg(&mut self, cfg_dir: &str) -> Result<(), std::io::Error> {
        let filepaths = match get_files_in_directory(cfg_dir) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}", e.to_string());
                return Err(e);
            }
        };

        for filepath in filepaths {
            let curdir = env::current_dir()?;

            let cfg_s = std::fs::read_to_string(format!("{}/{}", cfg_dir, filepath))?;
            let cfg: NSysCfg = match toml::from_str(&cfg_s) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{}", e.to_string());
                    continue;
                }
            };

            match cfg.functions {
                Some(ref v) => {
                    for key in v {
                        self.ncall_codes
                            .insert(key.1.ncall_code, (self.libs.len(), key.1.clone()));
                    }
                }
                None => {}
            }

            let cfg_clone = cfg.clone();
            let lib_filename: String = match self.platform {
                NSysOS::Linux => match cfg_clone.lib_filename_linux {
                    Some(v) => v,
                    None => {
                        eprintln!(
                            "Can't get config for {} library for this platform",
                            cfg_clone.name
                        );
                        "".to_string()
                    }
                },
                NSysOS::MacOS => match cfg_clone.lib_filename_linux {
                    Some(v) => v,
                    None => {
                        eprintln!(
                            "Can't get config for {} library for this platform",
                            cfg_clone.name
                        );
                        "".to_string()
                    }
                },
                NSysOS::Windows => match cfg_clone.lib_filename_linux {
                    Some(v) => v,
                    None => {
                        eprintln!(
                            "Can't get config for {} library for this platform",
                            cfg.name
                        );
                        "".to_string()
                    }
                },
            };

            match self.loadname(&lib_filename, cfg) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e.to_string());
                    continue;
                }
            };
        }

        Ok(())
    }

    pub fn call_code(
        &mut self,
        call_code: u16,
        args: &[VMValue],
        argc: u32,
    ) -> Result<VMValue, NSysError> {
        let funcdat = match self.ncall_codes.get(&call_code) {
            Some(v) => v,
            None => {
                eprintln!("No such callcode!");
                return Err(NSysError::InvalidCallCode(call_code));
            }
        };

        let lib: &mut NativeLibrary = match self.libs.get_mut(funcdat.0) {
            Some(v) => v,
            None => {
                return Err(NSysError::NoLibrary());
            }
        };
        let f = funcdat.1.clone();

        if (args.len() <= f.argc) {
            eprintln!("Invalid args!");
            return Err(NSysError::InvalidArgs());
        }
        let res = lib.call_foo(f.name, &args[1..f.argc], f.argc as u32); // r0 is for res

        match res {
            Ok(v) => {
                return Ok(v);
            }
            Err(e) => {
                return Err(NSysError::Libloading(e));
            }
        }
    }

    fn loadname(&mut self, filename: &str, cfg: NSysCfg) -> Result<(), String> {
        match NativeLibrary::new(filename, cfg) {
            Ok(nl) => {
                self.libs.push(nl);
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

#[derive(Debug)]
pub enum NSysError {
    Libloading(libloading::Error),
    fs(io::Error),
    InvalidCallCode(u16),
    NoLibrary(),
    InvalidArgs(),
    Other(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct NSysCfg {
    name: String,
    version: Option<String>,

    lib_filename_linux: Option<String>,
    lib_filename_macos: Option<String>,
    lib_filename_win: Option<String>,

    functions: Option<HashMap<String, NFuncCfg>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NFuncCfg {
    name: String,
    ncall_code: u16,
    argc: usize,
}

#[derive(Debug)]
pub enum NSysOS {
    Linux,
    MacOS,
    Windows,
}

type NativeFunction = unsafe extern "C" fn(
    vm_context: *mut std::ffi::c_void,
    registers: *mut u64,
    register_count: usize,
    args: *const u32,
    args_count: usize,
    result_register: usize,
);

#[derive(Debug)]
pub struct NativeLibrary {
    library: Library,
    conf: NSysCfg,
}

impl NativeLibrary {
    pub fn new(filename: &str, cfg: NSysCfg) -> Result<NativeLibrary, String> {
        let mut lib: Library;
        unsafe {
            lib = match Library::new(filename) {
                Ok(l) => l,
                Err(e) => {
                    return Err(e.to_string());
                }
            };
        }

        let res = NativeLibrary {
            library: (lib),
            conf: cfg,
        };
        Ok(res)
    }

    pub fn call_foo(
        &mut self,
        name: String,
        args: &[VMValue],
        argc: u32,
    ) -> Result<VMValue, libloading::Error> {
        let symb: Symbol<VMFFIFunction> = unsafe { self.library.get(name.as_bytes())? };
        let res = unsafe { symb(args.as_ptr(), argc) };

        Ok(res)
    }
}

fn get_files_in_directory(path: &str) -> std::io::Result<Vec<String>> {
    let entries = std::fs::read_dir(path)?;
    let files = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                path.file_name()?.to_str().map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();
    Ok(files)
}
