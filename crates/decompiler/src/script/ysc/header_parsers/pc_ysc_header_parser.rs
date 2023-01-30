use binary_layout::define_layout;
use binary_reader::{BinaryReader, Endian};

use crate::script::{YscHeaderParser, YscScriptHeader};

use super::read_pointer::ReadPointer;

define_layout!(pc_header, LittleEndian, {
  magic: u32, // 0x00
  _pad0x04: [u8; 4], // 0x04
  sub_header: u32, // 0x08
  _pad0x0c: [u8; 4], // 0x0C
  code_blocks_offset: u32, //0x10
  _pad0x14: [u8; 4], // 0x14
  globals_version: u32, // 0x18
  code_size: u32, // 0x1C
  parameter_count: u32, // 0x20
  statics_count: u32, // 0x24
  globals_count: u32, // 0x28
  natives_count: u32, // 0x2C
  statics_offset: u32, // 0x30
  _pad0x34: [u8; 4], // 0x34
  globals_offset: u32, // 0x38
  _pad0x3c: [u8; 4], // 0x3C
  natives_offset: u32, // 0x40
  _pad0x44: [u8; 4], // 0x44
  unk1: u32, // 0x48
  _pad0x4c: [u8; 4], // 0x4C
  unk2: u32, // 0x50
  _pad0x54: [u8; 4], // 0x54
  name_hash: u32, // 0x58
  unk3: u32, // 0x5C
  script_name_offset: u32, // 0x60
  _pad0x64: [u8; 4], // 0x64
  strings_offset: u32, // 0x68
  _pad0x6c: [u8; 4], // 0x6C
  strings_size: u32, // 0x70
  _pad0x74: [u8; 4], // 0x74
  unk4: u32, // 0x78
  _pad0x7c: [u8; 4] // 0x7C
});

pub struct PcYscHeaderParser;

impl YscHeaderParser for PcYscHeaderParser {
  fn parse(&self, bytes: &[u8]) -> anyhow::Result<YscScriptHeader> {
    let mut reader = BinaryReader::from_u8(bytes);
    reader.set_endian(Endian::Little);

    let rsc7_offset = ({
      reader.jmp(0);
      reader.read_u32()?
    } == 0x37435352)
      .then_some(0x10u32);

    let offset = rsc7_offset.unwrap_or_default();

    reader.jmp(offset as usize);

    let pc_header = pc_header::View::new(reader.read_bytes(pc_header::SIZE.unwrap())?.to_vec());

    reader.jmp((offset + pc_header.strings_offset().read_as_pointer()) as usize);
    let string_blocks = (pc_header.strings_size().read() + 0x3FFF) >> 14;
    let string_table_offsets = (0..string_blocks)
      .map(|_| {
        let res = reader.read_u32().map(|v| (v & 0xFFFFFF) + offset);
        reader.adv(4);
        res
      })
      .collect::<Result<_, _>>()?;

    reader.jmp((offset + pc_header.code_blocks_offset().read_as_pointer()) as usize);
    let code_blocks = (pc_header.code_size().read() + 0x3FFF) >> 14;
    let code_table_offsets = (0..code_blocks)
      .map(|_| {
        let res = reader.read_u32().map(|v| (v & 0xFFFFFF) + offset);
        reader.adv(4);
        res
      })
      .collect::<Result<_, _>>()?;

    Ok(YscScriptHeader {
      magic: pc_header.magic().read(),
      sub_header: pc_header.sub_header().read_as_pointer(),
      code_blocks_offset: pc_header.code_blocks_offset().read_as_pointer(),
      globals_version: pc_header.globals_version().read(),
      code_size: pc_header.code_size().read(),
      parameter_count: pc_header.parameter_count().read(),
      statics_count: pc_header.statics_count().read(),
      globals_count: pc_header.globals_count().read(),
      natives_count: pc_header.natives_count().read(),
      statics_offset: pc_header.statics_offset().read_as_pointer(),
      globals_offset: pc_header.globals_offset().read_as_pointer(),
      natives_offset: pc_header.natives_offset().read_as_pointer(),
      name_hash: pc_header.name_hash().read(),
      script_name_offset: pc_header.script_name_offset().read_as_pointer(),
      string_offset: pc_header.strings_offset().read_as_pointer(),
      strings_size: pc_header.strings_size().read(),
      rsc7_offset,
      string_table_offsets,
      code_table_offsets,
      string_blocks,
      code_blocks,
      script_name: {
        reader.jmp(offset as usize + pc_header.script_name_offset().read_as_pointer() as usize);
        let mut name = Vec::default();
        loop {
          let char = reader.read_u8()?;
          if char == 0x00 || char == 0xFF {
            break;
          }
          name.push(char)
        }
        String::from_utf8(name)?
      }
    })
  }
}
