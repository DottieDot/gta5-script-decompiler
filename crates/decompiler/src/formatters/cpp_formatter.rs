use std::collections::HashMap;

use itertools::Itertools;

use crate::decompiler::{
  decompiled::{DecompiledFunction, Statement, StatementInfo},
  BinaryOperator, CaseValue, Function, StackEntry, Type, UnaryOperator
};

use super::code_builder::CodeBuilder;

pub struct CppFormatter<'f, 'i, 'b> {
  functions: &'f HashMap<usize, Function<'i, 'b>>
}

impl<'f, 'i, 'b> CppFormatter<'f, 'i, 'b> {
  pub fn new(functions: &'f HashMap<usize, Function<'i, 'b>>) -> Self {
    Self { functions }
  }

  pub fn format_function(&self, function: &DecompiledFunction) -> String {
    let mut builder = CodeBuilder::default();

    builder
      .line(&self.create_signature(function))
      .line("{")
      .branch(|builder| {
        for statement in &function.statements {
          self.write_statement(statement, function, builder);
        }
      })
      .line("}");

    builder.collect()
  }

  fn create_signature(&self, function: &DecompiledFunction) -> String {
    let mut args = vec![];
    for i in 0..function.params {
      args.push(format!("parameter_{i}"));
    }
    format!("func {}({})", function.name, args.join(", "))
  }

  fn write_statement(
    &self,
    statement: &StatementInfo,
    function: &DecompiledFunction,
    builder: &mut CodeBuilder
  ) {
    match &statement.statement {
      Statement::Nop => {}
      Statement::Assign {
        destination,
        source
      } => {
        builder.line(&format!(
          "{destination} = {source};",
          destination = self.format_stack_entry(destination, function),
          source = self.format_stack_entry(source, function)
        ));
      }
      Statement::Return { values } => {
        match &values[..] {
          [single] => {
            builder.line(&format!(
              "return {};",
              self.format_stack_entry(single, function)
            ));
          }
          [] => {}
          _ => {}
        }
      }
      Statement::Throw { value } => {
        builder.line(&format!(
          "throw {};",
          self.format_stack_entry(value, function)
        ));
      }
      Statement::FunctionCall {
        args,
        function_address
      } => {
        builder.line(&format!(
          "{};",
          self.format_function_call(*function_address, args, function)
        ));
      }
      Statement::NativeCall { args, native_hash } => {
        builder.line(&format!(
          "{};",
          self.format_native_call(*native_hash, args, function)
        ));
      }
      Statement::If { condition, then } => {
        builder
          .line(&format!(
            "if ({})",
            self.format_stack_entry(condition, function)
          ))
          .line("{")
          .branch(|builder| {
            for statement in then {
              self.write_statement(statement, function, builder);
            }
          })
          .line("}");
      }
      Statement::IfElse {
        condition,
        then,
        els
      } => {
        builder
          .line(&format!(
            "if ({})",
            self.format_stack_entry(condition, function)
          ))
          .line("{")
          .branch(|builder| {
            for statement in then {
              self.write_statement(statement, function, builder);
            }
          })
          .line("}")
          .line("else")
          .line("{")
          .branch(|builder| {
            for statement in els {
              self.write_statement(statement, function, builder);
            }
          })
          .line("}");
      }
      Statement::WhileLoop { condition, body } => {
        builder
          .line(&format!(
            "while ({})",
            self.format_stack_entry(condition, function)
          ))
          .line("{")
          .branch(|builder| {
            for statement in body {
              self.write_statement(statement, function, builder);
            }
          })
          .line("}");
      }
      Statement::Switch { condition, cases } => {
        builder
          .line(&format!(
            "switch ({})",
            self.format_stack_entry(condition, function)
          ))
          .line("{")
          .branch(|builder| {
            for (body, case_values) in cases {
              for case in case_values {
                match case {
                  CaseValue::Value(val) => builder.line(&format!("case {val}:")),
                  CaseValue::Default => builder.line("default:")
                };
              }
              builder.branch(|builder| {
                for statement in body {
                  self.write_statement(statement, function, builder);
                }
              });
            }
          })
          .line("}");
      }
      Statement::Break => {
        builder.line("break;");
      }
      Statement::Continue => {
        builder.line("continue;");
      }
    }
  }

  fn format_stack_entry(&self, value: &StackEntry, function: &DecompiledFunction) -> String {
    match value {
      StackEntry::Int(i) => i.to_string(),
      StackEntry::Float(f) => f.to_string(),
      StackEntry::String(usize) => format!("STRING({usize})"),
      StackEntry::Struct { values } => {
        let values = values
          .iter()
          .map(|se| self.format_stack_entry(se, function))
          .join(", ");
        format!("({values})")
      }
      StackEntry::StructField { source, field } => {
        if let StackEntry::Deref(deref) = source.as_ref() {
          match deref.as_ref() {
            StackEntry::LocalRef(local) => {
              return format!("{}->f_{field}", self.format_local(*local, function))
            }
            StackEntry::StaticRef(stat) => return format!("static_{stat}->f_{field}"),
            StackEntry::GlobalRef(global) => return format!("global_{global}->f_{field}"),
            StackEntry::Ref(rf) => {
              return format!("{}->f_{field}", self.format_stack_entry(rf, function))
            }
            _ => {}
          }
        }
        format!("{}.f_{field}", self.format_stack_entry(source, function))
      }
      StackEntry::Offset { source, offset } => {
        match source.as_ref() {
          StackEntry::LocalRef(local) => {
            format!(
              "&({}->f_{})",
              self.format_local(*local, function),
              self.format_stack_entry(offset, function)
            )
          }
          StackEntry::StaticRef(stat) => {
            format!(
              "&(static_{stat}->f_{})",
              self.format_stack_entry(offset, function)
            )
          }
          StackEntry::GlobalRef(global) => {
            format!(
              "&(global_{global}->f_{})",
              self.format_stack_entry(offset, function)
            )
          }
          StackEntry::Ref(rf) => {
            format!(
              "&({}->f_{})",
              self.format_stack_entry(rf, function),
              self.format_stack_entry(offset, function)
            )
          }
          _ => {
            format!(
              "&({}->f_{})",
              self.format_stack_entry(source, function),
              self.format_stack_entry(offset, function)
            )
          }
        }
      }
      StackEntry::ArrayItem {
        source,
        index,
        item_size
      } => {
        let source = match source.as_ref() {
          StackEntry::LocalRef(local) => self.format_local(*local, function),
          StackEntry::StaticRef(stat) => format!("static_{stat}"),
          StackEntry::GlobalRef(stat) => format!("global_{stat}"),
          StackEntry::Ref(stat) => self.format_stack_entry(stat, function),
          other => self.format_stack_entry(other, function)
        };
        format!(
          "{}[{} /* {item_size} */]",
          source,
          self.format_stack_entry(index, function)
        )
      }
      StackEntry::LocalRef(local) => format!("&{}", self.format_local(*local, function)),
      StackEntry::StaticRef(stat) => format!("&static_{stat}"),
      StackEntry::GlobalRef(global) => format!("&global_{global}"),
      StackEntry::Deref(deref) => {
        match deref.as_ref() {
          StackEntry::LocalRef(local) => self.format_local(*local, function),
          StackEntry::StaticRef(stat) => format!("static_{stat}"),
          StackEntry::GlobalRef(global) => format!("global_{global}"),
          StackEntry::ArrayItem { .. } => self.format_stack_entry(deref, function).to_owned(),
          StackEntry::Offset { source, offset } => {
            match source.as_ref() {
              StackEntry::LocalRef(local) => {
                format!(
                  "{}->f_{}",
                  self.format_local(*local, function),
                  self.format_stack_entry(offset, function)
                )
              }
              StackEntry::StaticRef(stat) => {
                format!(
                  "static_{stat}->f_{}",
                  self.format_stack_entry(offset, function)
                )
              }
              StackEntry::GlobalRef(global) => {
                format!(
                  "global_{global}->f_{}",
                  self.format_stack_entry(offset, function)
                )
              }
              StackEntry::ArrayItem { .. } => {
                format!(
                  "{}.f_{}",
                  self.format_stack_entry(source, function),
                  self.format_stack_entry(offset, function)
                )
              }
              StackEntry::Ref(rf) => {
                format!(
                  "{}->f_{}",
                  self.format_stack_entry(rf, function),
                  self.format_stack_entry(offset, function)
                )
              }
              _ => {
                format!(
                  "{}->f_{}",
                  self.format_stack_entry(source, function),
                  self.format_stack_entry(offset, function)
                )
              }
            }
          }
          _ => format!("*({})", self.format_stack_entry(deref, function))
        }
      }
      StackEntry::Ref(rf) => format!("&{}", self.format_stack_entry(rf, function)),
      StackEntry::CatchValue => todo!(),
      StackEntry::BinaryOperator { lhs, rhs, op, .. } => {
        // TODO: Braces
        let op = match op {
          BinaryOperator::Add => "+",
          BinaryOperator::Subtract => "-",
          BinaryOperator::Multiply => "*",
          BinaryOperator::Divide => "/",
          BinaryOperator::BitwiseAnd => "&",
          BinaryOperator::BitwiseOr => "|",
          BinaryOperator::BitwiseXor => "^",
          BinaryOperator::Modulo => "%",
          BinaryOperator::Equal => "==",
          BinaryOperator::NotEqual => "!=",
          BinaryOperator::GreaterThan => ">",
          BinaryOperator::GreaterOrEqual => ">=",
          BinaryOperator::LowerThan => "<",
          BinaryOperator::LowerOrEqual => "<=",
          BinaryOperator::BitTest => {
            return format!(
              "BitTest({lhs}, {rhs})",
              lhs = self.format_stack_entry(lhs, function),
              rhs = self.format_stack_entry(rhs, function)
            )
          }
        };

        format!(
          "{lhs} {op} {rhs}",
          lhs = self.format_stack_entry(lhs, function),
          rhs = self.format_stack_entry(rhs, function)
        )
      }
      StackEntry::UnaryOperator { lhs, op, .. } => {
        let op = match op {
          UnaryOperator::Not => "!",
          UnaryOperator::Negate => "-"
        };

        format!("{op}{}", self.format_stack_entry(lhs, function))
      }
      StackEntry::Cast { source, ty } => {
        let ty = match ty {
          Type::Int => "int",
          Type::Float => "float",
          Type::Bool => "bool",
          Type::String | Type::Pointer(_) | Type::Array(_, _) | Type::Struct | Type::Unknown => {
            panic!("unsupported cast")
          }
        };
        format!("({ty}){}", self.format_stack_entry(source, function))
      }
      StackEntry::StringHash(str) => format!("HASH({})", self.format_stack_entry(str, function)),
      StackEntry::FunctionCallResult {
        args,
        function_address,
        ..
      } => self.format_function_call(*function_address, args, function),
      StackEntry::NativeCallResult {
        args, native_hash, ..
      } => self.format_native_call(*native_hash, args, function)
    }
  }

  fn format_function_call(
    &self,
    address: usize,
    args: &Vec<StackEntry>,
    function: &DecompiledFunction
  ) -> String {
    let args = args
      .iter()
      .map(|arg| format!("{}", self.format_stack_entry(arg, function)))
      .join(", ");
    let function = self
      .functions
      .get(&address)
      .map(|f| f.name.clone())
      .unwrap_or_else(|| format!("unk_fn{address:08X}"));
    format!("{function}({args})")
  }

  fn format_native_call(
    &self,
    native_hash: u64,
    args: &Vec<StackEntry>,
    function: &DecompiledFunction
  ) -> String {
    let args = args
      .iter()
      .map(|arg| format!("{}", self.format_stack_entry(arg, function)))
      .join(", ");
    format!("unk_0x{native_hash:016X}({args})")
  }

  fn format_local(&self, local: usize, function: &DecompiledFunction) -> String {
    if local < function.params {
      format!("parameter_{local}")
    } else {
      format!("local_{}", local - function.params)
    }
  }
}
