use std::cmp::Ordering;

pub fn bubble_sort_by<T>(data: &mut [T], compare: impl Fn(&T, &T) -> Ordering) {
  let len = data.len();
  let mut swapped = true;
  while swapped {
    for i in 0..len - 1 {
      swapped = false;
      if let Ordering::Greater = compare(&data[i], &data[i + 1]) {
        data.swap(i, i + 1);
        swapped = true;
      }
    }
  }
}
