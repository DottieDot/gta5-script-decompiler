use std::cmp::Ordering;

pub fn try_bubble_sort_by<T, E>(
  data: &mut [T],
  compare: impl Fn(&T, &T) -> Result<Ordering, E>
) -> Result<(), E> {
  let len = data.len();
  let mut swapped = true;
  while swapped {
    for i in 0..len - 1 {
      swapped = false;
      if let Ordering::Greater = compare(&data[i], &data[i + 1])? {
        data.swap(i, i + 1);
        swapped = true;
      }
    }
  }
  Ok(())
}
