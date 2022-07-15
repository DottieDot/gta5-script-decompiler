use binary_layout::{Endianness, FieldView, PrimitiveField};

pub(crate) trait ReadPointer {
  fn read_as_pointer(&self) -> u32;
}

impl<S: AsRef<[u8]>, E: Endianness, const OFFSET_: usize> ReadPointer
  for FieldView<S, PrimitiveField<u32, E, OFFSET_>>
{
  fn read_as_pointer(&self) -> u32 {
    self.read() & 0xFFFFFF
  }
}
