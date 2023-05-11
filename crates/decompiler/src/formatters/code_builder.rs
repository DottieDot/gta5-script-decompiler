#[derive(Default)]
pub struct CodeBuilder {
  code:          String,
  indent:        u32,
  indent_string: String
}

impl CodeBuilder {
  pub fn collect(self) -> String {
    self.code
  }

  pub fn code(&mut self, text: &str) -> &mut Self {
    self.code.push_str(&text.replace("\n", &self.indent_string));
    self
  }

  pub fn line(&mut self, text: &str) -> &mut Self {
    self.code.push_str(&self.indent_string);
    self.code.push_str(text);
    self.code.push_str("\n");
    self
  }

  pub fn branch(&mut self, cb: impl Fn(&mut Self)) -> &mut Self {
    self.push_indent();
    cb(self);
    self.pop_indent();
    self
  }

  fn push_indent(&mut self) {
    self.indent += 1;
    self.indent_string = "\t".repeat(self.indent as usize);
  }

  fn pop_indent(&mut self) {
    self.indent -= 1;
    self.indent_string = "\t".repeat(self.indent as usize);
  }
}
