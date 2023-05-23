use std::collections::HashMap;

use crate::resources::{CrossMap, Natives};

use super::{Function, ScriptGlobals, ScriptStatics};

pub struct DecompilerData<'i, 'b> {
  pub statics:   ScriptStatics,
  pub globals:   ScriptGlobals,
  pub natives:   Natives,
  pub cross_map: CrossMap,
  pub functions: HashMap<usize, Function<'i, 'b>>
}
