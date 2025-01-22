use std::{collections::HashMap, error::Error, fs, path::PathBuf, time::Duration};

use anyhow::Context;
use clap::Parser;
use glob::glob;
use gta5_script_decompiler::{
  decompiler::{get_functions, DecompilerData, ScriptGlobals, ScriptStatics},
  disassembler::disassemble,
  formatters::{AssemblyFormatter, CppFormatter},
  resources::{CrossMap, Natives},
  script::parse_ysc_file
};
use indicatif::{ProgressBar, ProgressStyle};

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), anyhow::Error>
where
  T: std::str::FromStr,
  T::Err: Error + 'static + Send + Sync,
  U: std::str::FromStr,
  U::Err: Error + 'static + Send + Sync
{
  let pos = s
    .find(':')
    .ok_or_else(|| anyhow::format_err!("invalid KEY:value: no `:` found in `{}`", s))?;
  Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

/// A YSC Decompiler for Grand Theft Auto 5
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
  /// YSC input glob pattern
  #[arg(short, long)]
  input: String,

  /// Output directory
  #[arg(short, long)]
  output: PathBuf,

  /// crossmap.json file override
  #[arg(short, long)]
  xmap: Option<PathBuf>,

  /// natives.json file override
  #[arg(short, long)]
  natives: Option<PathBuf>,

  /// A comma separated list of functions to generate function graphs for
  /// The functions should be formatted as a key-value pair indicating the script, and the function index
  /// Example: freemode:123,abigail:10
  #[arg(short, long, value_delimiter = ',', value_parser = parse_key_val::<String, usize>, verbatim_doc_comment)]
  graphs: Option<Vec<(String, usize)>>,

  /// Enables disassembly output
  #[arg(short, long, default_value_t = false)]
  disassemble: bool,

  /// Number of bytes to show in disassembly outputs
  #[arg(short, long, default_value_t = 0)]
  bytes: usize,

  /// Show addresses in disassembly outputs
  #[arg(short, long, default_value_t = false)]
  addresses: bool
}

fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  let globals = ScriptGlobals::default();
  let natives =
    Natives::from_json_file("./resources/natives.json").context("could not open natives.json")?;
  let cross_map = CrossMap::from_json_file("./resources/crossmap.json")
    .context("could not open crossmap.json")?;

  let script_files = glob(&args.input)?
    .filter_map(|file| file.ok())
    .collect::<Vec<_>>();

  let pb = ProgressBar::new(script_files.len().try_into().unwrap());
  pb.set_style(
    ProgressStyle::with_template(
      "{spinner:.green} [{elapsed_precise}] [{bar:40.blue}] {pos}/{len} {msg}"
    )
    .unwrap()
  );
  pb.enable_steady_tick(Duration::from_millis(50));

  for file in &script_files {
    pb.set_message("");

    let script = parse_ysc_file(&file)?;

    pb.set_message(script.header.name.clone());

    let disassembly = disassemble(&script.code)?;

    let output_folder = args.output.join(&script.header.name);

    fs::create_dir_all(&output_folder)?;

    let assembly_formatter =
      AssemblyFormatter::new(&disassembly, args.addresses, args.bytes, &script.strings);

    if args.disassemble {
      let disassembly = assembly_formatter.format(&disassembly, true);
      let output_file = format!("{}.scasm", script.header.name);

      fs::write(output_folder.join(output_file), disassembly)?;
    }

    let statics = ScriptStatics::new(script.header.static_count.try_into().unwrap());

    let functions = get_functions(&disassembly);
    let function_map = functions
      .iter()
      .map(|f| (f.location, f.clone()))
      .collect::<HashMap<_, _>>();

    if let Some(graphs) = &args.graphs {
      let functions_to_generate_graphs_for = graphs
        .iter()
        .filter_map(|(name, function)| (script.header.name == *name).then_some(*function));

      for function_index in functions_to_generate_graphs_for {
        if let Some(function) = functions.get(function_index) {
          let dot = function.dot_string(&assembly_formatter);
          let output_file = format!("{}.dot", function.name);

          fs::write(output_folder.join(output_file), dot)?;
        }
      }
    }
    let data = DecompilerData {
      statics:   &statics,
      globals:   &globals,
      natives:   &natives,
      cross_map: &cross_map,
      functions: &function_map
    };

    let decompiled = functions
      .iter()
      .filter_map(|func| {
        match func.decompile(&script, &data) {
          Ok(d) => Some(d),
          Err(_) => None
        }
      })
      .collect::<Vec<_>>();

    let cpp_formatter = CppFormatter::new(data);

    let code = decompiled
      .iter()
      .map(|func| cpp_formatter.format_function(func))
      .collect::<Vec<_>>()
      .join("\n");

    let output_file = format!("{}.cpp", script.header.name);

    fs::write(output_folder.join(output_file), code)?;

    pb.inc(1);
  }
  pb.finish_with_message(format!("Decompiled {} scripts", script_files.len()));

  Ok(())
}
