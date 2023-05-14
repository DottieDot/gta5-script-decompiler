use std::{collections::HashMap, fs};

use gta5_script_decompiler::{
  decompiler::functions,
  disassembler::disassemble,
  formatters::{AssemblyFormatter, CppFormatter},
  script::parse_ysc_file
};

fn main() -> anyhow::Result<()> {
  let script = parse_ysc_file(r"./input.ysc")?;

  let disassembly = disassemble(&script.code)?;

  //  let formatter = AssemblyFormatter::new(&disassembly, false, 0, &script.strings);

  // let output = formatter.format(&disassembly, true);

  // fs::write("output.scasm", output)?;

  let functions = functions(&disassembly);
  let function_map = functions
    .clone()
    .into_iter()
    .map(|f| (f.location, f))
    .collect::<HashMap<_, _>>();

  // *WORKS:
  // - 2243
  // - 2262 (&&)
  // - 2263 (&& else if)
  // - 2283 (|| else if)
  // - 1191 (&& and ||)
  // - 1271 (lots of && and ||)
  // - 2229 (while & break)
  // - 6294 (switch)
  // - 686 (disconnected loop)
  // TODO:
  let func = &functions[1271];
  let dot = func.dot_string(AssemblyFormatter::new(
    &disassembly,
    false,
    0,
    &script.strings
  ));

  fs::write("output.dot", dot)?;

  let flow = func.graph.reconstruct_control_flow();
  fs::write("flow.rs", format!("{flow:#?}"))?;

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

  let cpp_formatter = CppFormatter::new(&function_map);
  let decompiled = func.decompile(&script, &function_map)?;
  let formatted = cpp_formatter.format_function(&decompiled);

  fs::write("output.rs", format!("{decompiled:#?}"))?;
  fs::write("output.cpp", formatted)?;

  Ok(())
}
