#![allow(dead_code)]

use anyhow::Result;
use duct::{Expression};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{PathBuf};
use std::str::from_utf8;
use thinp::file_utils;

pub mod thin_xml_generator;
pub mod cache_xml_generator;
pub mod test_dir;

use crate::common::thin_xml_generator::{write_xml, SingleThinS};
use test_dir::TestDir;

//------------------------------------------

// FIXME: write a macro to generate these commands
#[macro_export]
macro_rules! thin_check {
    ( $( $arg: expr ),* ) => {
        {
            use std::ffi::OsString;
            let args: &[OsString] = &[$( Into::<OsString>::into($arg) ),*];
            duct::cmd("bin/thin_check", args).stdout_capture().stderr_capture()
        }
    };
}

#[macro_export]
macro_rules! thin_restore {
    ( $( $arg: expr ),* ) => {
        {
            use std::ffi::OsString;
            let args: &[OsString] = &[$( Into::<OsString>::into($arg) ),*];
            duct::cmd("bin/thin_restore", args).stdout_capture().stderr_capture()
        }
    };
}

#[macro_export]
macro_rules! thin_dump {
    ( $( $arg: expr ),* ) => {
        {
            use std::ffi::OsString;
            let args: &[OsString] = &[$( Into::<OsString>::into($arg) ),*];
            duct::cmd("bin/thin_dump", args).stdout_capture().stderr_capture()
        }
    };
}

#[macro_export]
macro_rules! thin_rmap {
    ( $( $arg: expr ),* ) => {
        {
            use std::ffi::OsString;
            let args: &[OsString] = &[$( Into::<OsString>::into($arg) ),*];
            duct::cmd("bin/thin_rmap", args).stdout_capture().stderr_capture()
        }
    };
}

#[macro_export]
macro_rules! thin_repair {
    ( $( $arg: expr ),* ) => {
        {
            use std::ffi::OsString;
            let args: &[OsString] = &[$( Into::<OsString>::into($arg) ),*];
            duct::cmd("bin/thin_repair", args).stdout_capture().stderr_capture()
        }
    };
}

#[macro_export]
macro_rules! thin_delta {
    ( $( $arg: expr ),* ) => {
        {
            use std::ffi::OsString;
            let args: &[OsString] = &[$( Into::<OsString>::into($arg) ),*];
            duct::cmd("bin/thin_delta", args).stdout_capture().stderr_capture()
        }
    };
}

#[macro_export]
macro_rules! thin_metadata_pack {
    ( $( $arg: expr ),* ) => {
        {
            use std::ffi::OsString;
            let args: &[OsString] = &[$( Into::<OsString>::into($arg) ),*];
            duct::cmd("bin/thin_metadata_pack", args).stdout_capture().stderr_capture()
        }
    };
}

#[macro_export]
macro_rules! thin_metadata_unpack {
    ( $( $arg: expr ),* ) => {
        {
            use std::ffi::OsString;
            let args: &[OsString] = &[$( Into::<OsString>::into($arg) ),*];
            duct::cmd("bin/thin_metadata_unpack", args).stdout_capture().stderr_capture()
        }
    };
}

#[macro_export]
macro_rules! cache_check {
    ( $( $arg: expr ),* ) => {
        {
            use std::ffi::OsString;
            let args: &[OsString] = &[$( Into::<OsString>::into($arg) ),*];
            duct::cmd("bin/cache_check", args).stdout_capture().stderr_capture()
        }
    };
}

//------------------------------------------

// Returns stderr, a non zero status must be returned
pub fn run_fail(command: Expression) -> Result<String> {
    let output = command.stderr_capture().unchecked().run()?;
    assert!(!output.status.success());
    Ok(from_utf8(&output.stderr[0..]).unwrap().to_string())
}

pub fn mk_valid_xml(td: &mut TestDir) -> Result<PathBuf> {
    let xml = td.mk_path("meta.xml");
    let mut gen = SingleThinS::new(0, 1024, 2048, 2048);
    write_xml(&xml, &mut gen)?;
    Ok(xml)
}

pub fn mk_valid_md(td: &mut TestDir) -> Result<PathBuf> {
    let xml = td.mk_path("meta.xml");
    let md = td.mk_path("meta.bin");

    let mut gen = SingleThinS::new(0, 1024, 20480, 20480);
    write_xml(&xml, &mut gen)?;

    let _file = file_utils::create_sized_file(&md, 4096 * 4096);
    thin_restore!("-i", xml, "-o", &md).run()?;
    Ok(md)
}

pub fn mk_zeroed_md(td: &mut TestDir) -> Result<PathBuf> {
    let md = td.mk_path("meta.bin");
    eprintln!("path = {:?}", md);
    let _file = file_utils::create_sized_file(&md, 4096 * 4096);
    Ok(md)
}

pub fn accepts_flag(flag: &str) -> Result<()> {
    let mut td = TestDir::new()?;
    let md = mk_valid_md(&mut td)?;
    thin_check!(flag, &md).run()?;
    Ok(())
}

pub fn superblock_all_zeroes(path: &PathBuf) -> Result<bool> {
    let mut input = OpenOptions::new().read(true).write(false).open(path)?;
    let mut buf = vec![0; 4096];
    input.read_exact(&mut buf[0..])?;
    for b in buf {
        if b != 0 {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn damage_superblock(path: &PathBuf) -> Result<()> {
    let mut output = OpenOptions::new().read(false).write(true).open(path)?;
    let buf = [0u8; 512];
    output.write_all(&buf)?;
    Ok(())
}

//------------------------------------------