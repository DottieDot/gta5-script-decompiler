use std::fs;

use gta5_script_decompiler::{
  decompiler::function_dot_string, disassembler::disassemble, formatters::AssemblyFormatter,
  script::parse_ysc_file
};

fn main() -> anyhow::Result<()> {
  let script = parse_ysc_file(r"C:\Users\Tvang\Desktop\freemode_ysc\freemode.ysc.full")?;
  let disassembly = disassemble(&script.code)?;

  let formatter = AssemblyFormatter::new(&disassembly, true, 8);

  let output = formatter.format(&disassembly, true);

  fs::write("output.scasm", output)?;

  let dot = function_dot_string(
    &disassembly,
    19043,
    AssemblyFormatter::new(&disassembly, false, 0)
  );

  fs::write("output.dot", dot)?;

  Ok(())
}
