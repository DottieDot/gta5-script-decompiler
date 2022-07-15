use std::fs;

use crate::parser::ScriptHeader;

mod parser;

fn main() {
  let bytes = fs::read(
    "/Users/taranvangoenigen/Documents/Repositories/gta5-script-decompiler/abigail1.ysc.full"
  )
  .unwrap();
  let header = ScriptHeader::read_pc_header(&bytes);

  println!("{header:#?}");
}
