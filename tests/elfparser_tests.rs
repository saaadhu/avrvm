extern crate avrvm;

use avrvm::elfparser;

#[test]
fn elf_header_read() {
    let header = elfparser::read_elf_header("/home/saaadhu/code/personal/avrvm/tests/test.elf").unwrap();
    assert_eq!(header.class, elfparser::ElfClass::Bit32);
    assert_eq!(header.endianness, elfparser::ElfEndianness::Little);
    assert_eq!(header.ident_version, elfparser::ElfVersion::Current);
    assert_eq!(header.filetype, elfparser::ElfFileType::Executable);
    assert_eq!(header.machine, elfparser::ElfMachine::Processor(83));
    assert_eq!(header.version, elfparser::ElfVersion::Current);
    assert_eq!(header.entry, 0x0);
}
