use std::collections::HashMap;

use crate::resources::{CrossMap, Natives};

use super::{Function, ScriptGlobals, ScriptStatics};

#[derive(Clone, Copy)]
pub struct DecompilerData<'d, 'i, 'b> {
  pub statics:   &'d ScriptStatics,
  pub globals:   &'d ScriptGlobals,
  pub natives:   &'d Natives,
  pub cross_map: &'d CrossMap,
  pub functions: &'d HashMap<usize, Function<'i, 'b>>
}
