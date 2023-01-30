use std::fs;

use gta5_script_decompiler::{
  disassembler::disassemble, formatters::AssemblyFormatter, script::parse_ysc_file
};

fn main() -> anyhow::Result<()> {
  let script =
    parse_ysc_file(r"C:\Users\Tvang\Desktop\shop_controller_ysc\shop_controller.ysc.full")?;
  let disassembly = disassemble(&script.code)?;

  let formatter = AssemblyFormatter {
    include_offset:    true,
    max_bytes_to_show: 8
  };

  let output = formatter.format(&script.code, &disassembly);

  fs::write("output.txt", output)?;

  Ok(())
}
