use std::cmp;

pub fn flatten_table(
  bytes: &[u8],
  total_size: usize,
  block_offsets: &[usize],
  block_size: usize
) -> Vec<u8> {
  block_offsets
    .iter()
    .enumerate()
    .flat_map(|(index, offset)| {
      let to_take = cmp::min(total_size - index * block_size, block_size);
      let end_offset = *offset as usize + to_take;
      bytes[(*offset as usize)..end_offset].to_vec()
    })
    .collect::<Vec<_>>()
}
