use std::matches;

use itertools::Itertools;

use crate::decompiler::{
  decompiled::{DecompiledFunction, Statement, StatementInfo},
  BinaryOperator, CaseValue, DecompilerData, LinkedValueType, Primitives, StackEntry,
  StackEntryInfo, UnaryOperator, ValueType, ValueTypeInfo
};

use super::code_builder::CodeBuilder;

pub struct CppFormatter<'d, 'i, 'b> {
  data: DecompilerData<'d, 'i, 'b>
}

impl<'d, 'i, 'b> CppFormatter<'d, 'i, 'b> {
  pub fn new(data: DecompilerData<'d, 'i, 'b>) -> Self {
    Self { data }
  }

  pub fn format_function(&self, function: &DecompiledFunction) -> String {
    let mut builder = CodeBuilder::default();

    builder
      .line(&self.create_signature(function))
      .line("{")
      .branch(|builder| {
        self.declare_locals(function, builder);
        for statement in &function.statements {
          self.write_statement(statement, function, builder, false);
        }
      })
      .line("}");

    builder.collect()
  }

  fn create_signature(&self, function: &DecompiledFunction) -> String {
    let mut args = vec![];

    let mut iter = function.params.iter().enumerate();
    while let Some((i, p)) = iter.next() {
      args.push(format!(
        "{} {} /* {i} */",
        self.format_type(&p.borrow()),
        self.format_local(i, function)
      ));
      let _ = iter.advance_by(p.borrow().size() - 1);
    }
    format!(
      "{} {}({})",
      function
        .returns
        .as_ref()
        .map(|returns| self.format_type(&returns.borrow()))
        .unwrap_or("void".to_owned()),
      function.name,
      args.join(", ")
    )
  }

  fn declare_locals(&self, function: &DecompiledFunction, builder: &mut CodeBuilder) {
    let mut iter = function.locals.iter().enumerate();
    while let Some((i, p)) = iter.next() {
      builder.line(&format!(
        "{} {} /* {} */;",
        self.format_type(&p.borrow()),
        self.format_local(function.params.len() + 2 + i, function),
        function.params.len() + 2 + i
      ));
      let _ = iter.advance_by(p.borrow().size() - 1);
    }

    if !function.locals.is_empty() {
      builder.line("");
    }
  }

  fn write_statement(
    &self,
    statement: &StatementInfo,
    function: &DecompiledFunction,
    builder: &mut CodeBuilder,
    else_if: bool
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
          [] => {
            builder.line("return;");
          }
          values => {
            let values = values
              .iter()
              .map(|v| self.format_stack_entry(v, function))
              .join(", ");
            builder.line(&format!("return {{ {values} }}"));
          }
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
            "{}if ({})",
            if else_if { "else " } else { "" },
            self.format_stack_entry(condition, function)
          ))
          .line("{")
          .branch(|builder| {
            for statement in then {
              self.write_statement(statement, function, builder, false);
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
            "{}if ({})",
            if else_if { "else " } else { "" },
            self.format_stack_entry(condition, function)
          ))
          .line("{")
          .branch(|builder| {
            for statement in then {
              self.write_statement(statement, function, builder, false);
            }
          })
          .line("}");

        match &els[..] {
          [st @ StatementInfo {
            statement: Statement::IfElse { .. } | Statement::If { .. },
            ..
          }] => self.write_statement(st, function, builder, true),
          _ => {
            builder
              .line("else")
              .line("{")
              .branch(|builder| {
                for statement in els {
                  self.write_statement(statement, function, builder, false);
                }
              })
              .line("}");
          }
        }
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
              self.write_statement(statement, function, builder, false);
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
                  self.write_statement(statement, function, builder, false);
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
      Statement::StringCopy {
        destination,
        string,
        max_length
      } => {
        builder.line(&format!(
          "string_copy({}, {}, {max_length});",
          self.format_stack_entry(destination, function),
          self.format_stack_entry(string, function)
        ));
      }
      Statement::IntToString {
        destination,
        int,
        max_length
      } => {
        builder.line(&format!(
          "int_to_string({}, {}, {max_length});",
          self.format_stack_entry(destination, function),
          self.format_stack_entry(int, function)
        ));
      }
      Statement::StringConcat {
        destination,
        string,
        max_length
      } => {
        builder.line(&format!(
          "string_concat({}, {}, {max_length});",
          self.format_stack_entry(destination, function),
          self.format_stack_entry(string, function)
        ));
      }
      Statement::StringIntConcat {
        destination,
        int,
        max_length
      } => {
        builder.line(&format!(
          "string_int_concat({}, {}, {max_length});",
          self.format_stack_entry(destination, function),
          self.format_stack_entry(int, function)
        ));
      }
      Statement::MemCopy {
        destination,
        source,
        buffer_size,
        ..
      } => {
        builder.line(&format!(
          "mem_copy({}, {}, {});",
          self.format_stack_entry(destination, function),
          {
            match &source[..] {
              [] => "{}".to_owned(),
              [value] => self.format_stack_entry(value, function),
              values => {
                let joined = values
                  .iter()
                  .map(|value| self.format_stack_entry(value, function))
                  .join(", ");
                format!("{{ {joined} }}")
              }
            }
          },
          self.format_stack_entry(buffer_size, function)
        ));
      }
    }
  }

  fn format_stack_entry(&self, value: &StackEntryInfo, function: &DecompiledFunction) -> String {
    match &value.entry {
      StackEntry::Int(i) => i.to_string(),
      StackEntry::Float(f) => {
        if f.trunc() == *f {
          format!("{f}.f")
        } else {
          format!("{f}f")
        }
      }
      StackEntry::String(string) => format!("\"{string}\""),
      StackEntry::ResultStruct { values } => {
        let values = values
          .iter()
          .map(|se| self.format_stack_entry(se, function))
          .join(", ");
        format!("({values})")
      }
      StackEntry::StructField { source, field } => {
        if let StackEntry::Deref(deref) = &source.entry {
          if let StackEntry::Ref(rf) = &deref.entry {
            return format!("{}->f_{field}", self.format_stack_entry(rf, function));
          }
        }
        let ty = source.ty.borrow().get_concrete();
        if matches!(
          ty,
          ValueTypeInfo {
            ty: ValueType::Struct { .. },
            ..
          }
        ) {
          format!("{}.f_{field}", self.format_stack_entry(source, function))
        } else {
          format!("{}", self.format_stack_entry(source, function))
        }
      }
      StackEntry::Offset { source, offset } => {
        match &source.entry {
          StackEntry::Ref(rf) => {
            format!(
              "{}.f_{}",
              self.format_stack_entry(rf, function),
              self.format_stack_entry(offset, function)
            )
          }
          _ => {
            format!(
              "{}.f_{}",
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
        let source = match &source.entry {
          StackEntry::Ref(stat) => self.format_stack_entry(stat, function),
          _ => self.format_stack_entry(source, function)
        };
        format!(
          "{}[{} /* {item_size} */]",
          source,
          self.format_stack_entry(index, function)
        )
      }
      StackEntry::Local(local) => {
        format!("{}", self.format_local(*local, function))
      }
      StackEntry::Static(stat) => format!("static_{stat}"),
      StackEntry::Global(global) => format!("global_{global}"),
      StackEntry::Deref(deref) => {
        match &deref.entry {
          StackEntry::Ref(rf) => self.format_stack_entry(rf, function),
          _ => {
            format!("*({})", self.format_stack_entry(deref, function))
          }
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
          BinaryOperator::LogicalAnd => {
            match (&lhs.entry, &rhs.entry) {
              (
                StackEntry::BinaryOperator {
                  op: BinaryOperator::LogicalOr,
                  ..
                },
                StackEntry::BinaryOperator {
                  op: BinaryOperator::LogicalOr,
                  ..
                }
              ) => {
                return format!(
                  "({}) && ({})",
                  self.format_stack_entry(lhs, function),
                  self.format_stack_entry(rhs, function)
                );
              }
              (
                StackEntry::BinaryOperator {
                  op: BinaryOperator::LogicalOr,
                  ..
                },
                _
              ) => {
                return format!(
                  "({}) && {}",
                  self.format_stack_entry(lhs, function),
                  self.format_stack_entry(rhs, function)
                );
              }
              (
                _,
                StackEntry::BinaryOperator {
                  op: BinaryOperator::LogicalOr,
                  ..
                }
              ) => {
                return format!(
                  "{} && ({})",
                  self.format_stack_entry(lhs, function),
                  self.format_stack_entry(rhs, function)
                );
              }
              _ => "&&"
            }
          }
          BinaryOperator::LogicalOr => {
            match (&lhs.entry, &rhs.entry) {
              (
                StackEntry::BinaryOperator {
                  op: BinaryOperator::LogicalAnd,
                  ..
                },
                StackEntry::BinaryOperator {
                  op: BinaryOperator::LogicalAnd,
                  ..
                }
              ) => {
                return format!(
                  "({}) || ({})",
                  self.format_stack_entry(lhs, function),
                  self.format_stack_entry(rhs, function)
                );
              }
              (
                StackEntry::BinaryOperator {
                  op: BinaryOperator::LogicalAnd,
                  ..
                },
                _
              ) => {
                return format!(
                  "({}) || {}",
                  self.format_stack_entry(lhs, function),
                  self.format_stack_entry(rhs, function)
                );
              }
              (
                _,
                StackEntry::BinaryOperator {
                  op: BinaryOperator::LogicalAnd,
                  ..
                }
              ) => {
                return format!(
                  "{} || ({})",
                  self.format_stack_entry(lhs, function),
                  self.format_stack_entry(rhs, function)
                );
              }
              _ => "||"
            }
          }
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

        format!("{op}({})", self.format_stack_entry(lhs, function))
      }
      StackEntry::Cast { source } => {
        let ty = self.format_type(&value.ty.borrow());
        format!("({ty}){}", self.format_stack_entry(source, function))
      }
      StackEntry::StringHash(str) => {
        format!("HASH({})", self.format_stack_entry(str, function))
      }
      StackEntry::FunctionCallResult {
        args,
        function_address,
        ..
      } => self.format_function_call(*function_address, args, function),
      StackEntry::NativeCallResult {
        args, native_hash, ..
      } => self.format_native_call(*native_hash, args, function),
      StackEntry::Struct { origin, .. } => self.format_stack_entry(origin, function),
      StackEntry::FloatToVector(float) => {
        format!("F2V({})", self.format_stack_entry(float, function))
      }
    }
  }

  fn format_function_call(
    &self,
    address: usize,
    args: &[StackEntryInfo],
    function: &DecompiledFunction
  ) -> String {
    let args = args
      .iter()
      .map(|arg| format!("{}", self.format_stack_entry(arg, function)))
      .join(", ");
    let function = self
      .data
      .functions
      .get(&address)
      .map(|f| f.name.clone())
      .unwrap_or_else(|| format!("unk_fn{address:08X}"));
    format!("{function}({args})")
  }

  fn format_native_call(
    &self,
    native_hash: u64,
    args: &[StackEntryInfo],
    function: &DecompiledFunction
  ) -> String {
    let args = args
      .iter()
      .map(|arg| format!("{}", self.format_stack_entry(arg, function)))
      .join(", ");

    if let Some(native) = self.data.natives.get_native(native_hash) {
      format!("{}({args})", native.name)
    } else {
      format!("unk_0x{native_hash:016X}({args})")
    }
  }

  fn format_local(&self, local: usize, function: &DecompiledFunction) -> String {
    if local < function.params.len() {
      format!("parameter_{local}")
    } else {
      format!(
        "local_{}",
        local - function.params.len() - 2 /* return address and stack frame */
      )
    }
  }

  #[allow(clippy::only_used_in_recursion)]
  fn format_type(&self, ty: &LinkedValueType) -> String {
    let ty = ty.get_concrete();

    match &ty.ty {
      ValueType::Struct { fields } => {
        let fields = fields
          .iter()
          .map(|field| self.format_type(&field.borrow()))
          .join(", ");

        format!("struct<{fields}>")
      }
      ValueType::Array { item_type } => format!("{}[]", self.format_type(&item_type.borrow())),
      ValueType::Function { .. } => todo!(),
      ValueType::Primitive(primitive) => {
        match primitive {
          Primitives::Float => "float".to_owned(),
          Primitives::Int => "int".to_owned(),
          Primitives::String => "const char*".to_owned(),
          Primitives::Bool => "bool".to_owned(),
          Primitives::Unknown => "any".to_owned()
        }
      }
      ValueType::Ref(t) => format!("{}*", self.format_type(&t.borrow()))
    }
  }
}
