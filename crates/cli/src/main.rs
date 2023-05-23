use std::{collections::HashMap, fs, println};

use gta5_script_decompiler::{
  decompiler::{functions, DecompilerData, ScriptGlobals, ScriptStatics},
  disassembler::disassemble,
  formatters::{AssemblyFormatter, CppFormatter},
  resources::{CrossMap, Natives},
  script::parse_ysc_file
};

fn main() -> anyhow::Result<()> {
  let script = parse_ysc_file(r"./input.ysc")?;

  let disassembly = disassemble(&script.code)?;

  let statics = ScriptStatics::new(script.header.static_count as usize);
  let globals = ScriptGlobals::default();

  // let formatter = AssemblyFormatter::new(&disassembly, false, 0, &script.strings);

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
  let func = &functions[17672];
  let dot = func.dot_string(AssemblyFormatter::new(
    &disassembly,
    true,
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

  // {
  //   let func = &functions[812];
  //   func.decompile(&script, &function_map, &statics, &mut globals)?;
  // }
  // {
  //   let func = &functions[815];
  //   func.decompile(&script, &function_map, &statics, &mut globals)?;
  // }

  let data = DecompilerData {
    statics,
    globals,
    natives: Natives::from_json_file("./resources/natives.json")?,
    cross_map: CrossMap::from_json_file("./resources/crossmap.json")?,
    functions: function_map
  };

  let cpp_formatter = CppFormatter::new(&data);
  // let decompiled = func.decompile(&script, &function_map, &statics, &mut globals)?;
  // let formatted = cpp_formatter.format_function(&decompiled);

  let decompiled = functions[0..]
    .iter()
    .filter_map(|func| {
      // println!("{}", func.name);
      match func.decompile(&script, &data) {
        Ok(d) => Some(d),
        Err(e) => {
          println!("{e:#?}");
          None
        }
      }
    })
    .collect::<Vec<_>>();
  let code = decompiled
    .iter()
    .map(|func| {
      // println!("{}", func.name);
      cpp_formatter.format_function(func)
    })
    .collect::<Vec<_>>()
    .join("\n");

  //fs::write("output.rs", format!("{decompiled:#?}"))?;
  fs::write("output.cpp", code)?;

  // DEMO func_680

  Ok(())
}
