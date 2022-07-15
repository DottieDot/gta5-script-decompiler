use std::{fs, time::Instant};

use gta5_script_decompiler::parser::Script;

fn main() {
  let bytes = fs::read(
    "/Users/taranvangoenigen/Documents/Repositories/gta5-script-decompiler/abigail1.ysc.full"
  )
  .unwrap();

  let now = Instant::now();

  // Code block to measure.
  {
    let script = Script::from_pc_script(&bytes);
  }

  let elapsed = now.elapsed();
  println!("Elapsed: {:.2?}", elapsed);

  // println!("{script:#?}");

  // if let Ok(scr) = script {
  //   let assembly = parse_assembly(&scr.code);

  //   println!("{assembly:#?}");
  // }
}
