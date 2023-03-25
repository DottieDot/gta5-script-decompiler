use std::fs;

use gta5_script_decompiler::{
  disassembler::disassemble, formatters::AssemblyFormatter, script::parse_ysc_file
};

fn main() -> anyhow::Result<()> {
  let script = parse_ysc_file(r"C:\Users\Tvang\Desktop\freemode_ysc\freemode.ysc.full")?;
  let disassembly = disassemble(&script.code)?;

  let formatter = AssemblyFormatter::new(&disassembly, true, 8);

  let output = formatter.format(&disassembly, true);

  fs::write("output.scasm", output)?;

  Ok(())
}
