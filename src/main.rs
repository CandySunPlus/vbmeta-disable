use std::io::{Read, Seek};
use std::os::unix::fs::FileExt;
use std::{env, fs, io};

use enumflags2::{bitflags, BitFlags};

const AVB_MAGIC: &'static str = "AVB0";
const AVB_MAGIC_LEN: usize = 4;
const FLAG_OFFSET: u64 = 120;

#[bitflags]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
enum AvbVBMetaImageFlags {
    HashTreeDisabled = 1 << 0,
    VerificationDisabled = 1 << 1,
}

fn main() -> io::Result<()> {
    let arguments = env::args();

    let Some(vbmeta_file_name) = arguments.take(2).skip(1).next() else {
        panic!("Missing vbmeta file argument");
    };

    if vbmeta_file_name.starts_with("--help") || vbmeta_file_name.starts_with("-h") {
        println!("Usage: vbmeta-disable <vbmeta image>");
        return Ok(());
    }

    let mut vbmeta_file = fs::File::options()
        .read(true)
        .write(true)
        .open(vbmeta_file_name)?;

    let mut magic_buffer = [0u8; AVB_MAGIC_LEN];
    let mut flags_buffer = [0u8; 4];

    vbmeta_file.read_exact(&mut magic_buffer)?;

    if magic_buffer != AVB_MAGIC.as_bytes() {
        panic!("Invalid vbmeta magic");
    }

    let seek_offset = vbmeta_file.seek(io::SeekFrom::Start(FLAG_OFFSET))?;

    if seek_offset != FLAG_OFFSET {
        panic!("Seek failed");
    }

    vbmeta_file.read_exact(&mut flags_buffer)?;

    let mut flags =
        BitFlags::<AvbVBMetaImageFlags>::from_bits(u32::from_be_bytes(flags_buffer)).unwrap();

    println!(
        "Before: [HashTreeDisabled: {}] [VerificationDisabled: {}]",
        flags.contains(AvbVBMetaImageFlags::HashTreeDisabled),
        flags.contains(AvbVBMetaImageFlags::VerificationDisabled)
    );

    flags |= AvbVBMetaImageFlags::HashTreeDisabled | AvbVBMetaImageFlags::VerificationDisabled;

    vbmeta_file.write_at(&u32::to_be_bytes(flags.bits()), FLAG_OFFSET)?;

    println!(
        "After: [HashTreeDisabled: {}] [VerificationDisabled: {}]",
        flags.contains(AvbVBMetaImageFlags::HashTreeDisabled),
        flags.contains(AvbVBMetaImageFlags::VerificationDisabled)
    );

    Ok(())
}
