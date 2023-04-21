use std::fs;

use gta5_script_decompiler::{
  decompiler::{decompile_function, function_dot_string},
  disassembler::disassemble,
  formatters::AssemblyFormatter,
  script::parse_ysc_file
};

fn main() -> anyhow::Result<()> {
  let script = parse_ysc_file(r"./input.ysc")?;
  let disassembly = disassemble(&script.code)?;

  let formatter = AssemblyFormatter::new(&disassembly, false, 0, &script.strings);

  let output = formatter.format(&disassembly, true);

  fs::write("output.scasm", output)?;

  // let dot = function_dot_string(
  //   &disassembly,
  //   10,
  //   AssemblyFormatter::new(&disassembly, false, 0, &script.strings)
  // );

  // fs::write("output.dot", dot)?;

  /*
  18711:
    int func_18711()
    {
      return Global_1836354;
    }

  18796:
    bool func_18796()
    {
      return BitTest(func_20812(9904, -1, 0), 19);
    }

  118: !! THE CODE BELOW IS ACTUALLY "INCORRECT"
    bool func_118(var uParam0, var uParam1, var uParam2, var uParam3, var uParam4, var uParam5, var uParam6, var uParam7, var uParam8, var uParam9, var uParam10, var uParam11, var uParam12)
    {
      return NETWORK::NETWORK_IS_HANDLE_VALID(&uParam0, 13);
    }

  485:
    int func_485()
    {
      return func_481(MISC::GET_RANDOM_INT_IN_RANGE(0, 16));
    }
  */

  // let decompiled = decompile_function(&disassembly, &script, 485)?;

  // fs::write("output.rs", format!("{decompiled:#?}"))?;

  Ok(())
}
