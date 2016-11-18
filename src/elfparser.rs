use std::io::BufReader;
use std::io::Read;
use std::io::Result;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;

#[derive(Debug, PartialEq,Copy,Clone)]
pub enum ElfClass {
    NoClass = 0,
    Bit32 = 1,
    Bit64 = 2
}

#[derive(Debug, PartialEq,Copy,Clone)]
pub enum ElfEndianness {
    Unknown,
    Little,
    Big
}


#[derive(Debug, PartialEq,Copy,Clone)]
pub enum ElfVersion {
    Invalid,
    Current
}

#[derive(Debug, PartialEq,Copy,Clone)]
pub enum ElfFileType {
    Unknown,
    Relocatable,
    Executable,
    SharedObject,
    Core,
    ProcessorSpecific(u16)
}

#[derive(Debug, PartialEq,Copy,Clone)]
pub enum ElfMachine {
    NoMachine,
    M32,
    SPARC,
    I386,
    M68K,
    M88K,
    I860,
    MIPS,
    Processor(u16)
}


pub struct ElfHeader {
    pub class: ElfClass,
    pub endianness: ElfEndianness,
    pub ident_version: ElfVersion,
    pub filetype: ElfFileType,
    pub machine: ElfMachine,
    pub version: ElfVersion,
    pub entry: u32,
    pub phoff: u32,
    pub e_shoff: u32,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16
}

pub struct ProgramHeader {
}

pub struct ElfFile {
    Header: ElfHeader,
    ProgramHeader: ProgramHeader
}


fn get_elf_class (byte : u8) -> Result<ElfClass> {
    match byte {
        0u8 => Ok(ElfClass::NoClass),
        1u8 => Ok(ElfClass::Bit32),
        2u8 => Ok(ElfClass::Bit64),
        _ => Err(Error::new(ErrorKind::Other, "Unrecognized ElfClass"))
    }
}

fn get_elf_endianness (byte : u8) -> Result<ElfEndianness> {
    match byte {
        0u8 => Ok(ElfEndianness::Unknown),
        1u8 => Ok(ElfEndianness::Little),
        2u8 => Ok(ElfEndianness::Big),
        _ => Err(Error::new(ErrorKind::Other, "Unrecognized ElfEndianness"))
    }
}

fn get_elf_ident_version (byte : u8) -> Result<ElfVersion> {
    match byte {
        0u8 => Ok(ElfVersion::Invalid),
        1u8 => Ok(ElfVersion::Current),
        _ => Err(Error::new(ErrorKind::Other, "Unrecognized ElfVersion"))
    }
}

fn get_elf_filetype (data: u16) -> Result<ElfFileType> {
    match data {
        0 => Ok(ElfFileType::Unknown),
        1 => Ok(ElfFileType::Relocatable),
        2 => Ok(ElfFileType::Executable),
        3 => Ok(ElfFileType::SharedObject),
        4 => Ok(ElfFileType::Core),
        x @ 0xff00 ... 0xffff => Ok(ElfFileType::ProcessorSpecific(x)),
        _ => Err(Error::new(ErrorKind::Other, "Unrecognized ElfFileType"))
    }
}

fn get_elf_machine (data: u16) -> Result<ElfMachine> {
    match data {
        0 => Ok(ElfMachine::NoMachine),
        1 => Ok(ElfMachine::M32),
        2 => Ok(ElfMachine::SPARC),
        3 => Ok(ElfMachine::I386),
        4 => Ok(ElfMachine::M68K),
        5 => Ok(ElfMachine::M88K),
        6 => Ok(ElfMachine::I860),
        7 => Ok(ElfMachine::MIPS),
        x @ _ => Ok(ElfMachine::Processor(x))
    }
}

fn get_elf_version (data: u32) -> Result<ElfVersion> {
    match data {
        0 => Ok(ElfVersion::Invalid),
        1 => Ok(ElfVersion::Current),
        _ => Err(Error::new(ErrorKind::Other, "Unrecognized ElfVersion"))
    }
}

trait EndianAwareReader {
    fn read_u8(&mut self) -> Result<u8>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_u32(&mut self) -> Result<u32>;
}

struct ElfReader<'a> {
    inner : &'a mut Read,
    endianness : ElfEndianness
}

impl <'a> EndianAwareReader for ElfReader<'a> {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf :[u8;1] = [0; 1];
        try! (self.inner.read_exact(&mut buf));
        Ok(buf[0])
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buf: [u8;2] = [0; 2];
        try! (self.inner.read_exact(&mut buf));

        match self.endianness {
            ElfEndianness::Unknown => Err(Error::new(ErrorKind::Other, "Cannot proceed with unknown ElfEndianness")),
            ElfEndianness::Little => Ok((buf[1] as u16) << 8 | (buf[0] as u16)),
            ElfEndianness::Big => Ok((buf[0] as u16) << 8 | (buf[1] as u16))
        }
    }

    fn read_u32(&mut self) -> Result<u32> {
        let mut buf: [u8;4] = [0; 4];
        try! (self.inner.read_exact(&mut buf));

        match self.endianness {
            ElfEndianness::Unknown => Err(Error::new(ErrorKind::Other, "Cannot proceed with unknown ElfEndianness")),
            ElfEndianness::Little => Ok((buf[3] as u32) << 24 | (buf[2] as u32) << 16 | (buf[1] as u32) << 8 | buf[0] as u32),
            ElfEndianness::Big => Ok((buf[0] as u32) << 24 | (buf[1] as u32) << 16 | (buf[2] as u32) << 8 | buf[3] as u32)
        }
    }
}

pub fn read_elf_header (filename: &str) -> Result<ElfHeader> {
    let f = File::open(filename);
    let mut reader = BufReader::new(f.unwrap());

    let mut h = ElfHeader{
        class: ElfClass::NoClass,
        endianness: ElfEndianness::Unknown,
        ident_version: ElfVersion::Invalid,
        filetype: ElfFileType::Unknown,
        machine: ElfMachine::NoMachine,
        version: ElfVersion::Invalid,
        entry: 0,
        phoff: 0,
        e_shoff: 0,
        e_flags: 0,
        e_ehsize: 0,
        e_phentsize: 0,
        e_phnum: 0,
        e_shentsize: 0,
        e_shnum: 0,
        e_shstrndx: 0,

    };

    let mut id = [0; 16];
    try!(reader.read(&mut id));
    assert_eq! (id[0], 0x7F);
    assert_eq! (id[1], 'E' as u8);
    assert_eq! (id[2], 'L' as u8);
    assert_eq! (id[3], 'F' as u8);

    h.class = try!(get_elf_class(id[4] as u8));
    h.endianness = try!(get_elf_endianness(id[5] as u8));
    h.ident_version = try!(get_elf_ident_version(id[6] as u8));

    {
        let mut elfreader = ElfReader { inner : &mut reader, endianness : h.endianness};
        h.filetype = try!(elfreader.read_u16().and_then(get_elf_filetype));
        h.machine = try!(elfreader.read_u16().and_then(get_elf_machine));
        h.version = try!(elfreader.read_u32().and_then(get_elf_version));
        h.entry = try!(elfreader.read_u32());
        h.phoff = try!(elfreader.read_u32());
    }
    Ok(h)
}

