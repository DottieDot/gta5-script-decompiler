#[derive(Debug, Clone, Copy)]
pub struct ParentedList<'p, T> {
  last: Option<ParentNode<'p, T>>
}

impl<'p, T> ParentedList<'p, T> {
  #[must_use]
  pub fn with_appended<'s: 'p>(&'s self, value: T) -> ParentedList<'s, T> {
    Self {
      last: Some(ParentNode {
        parent: self.last.as_ref(),
        value
      })
    }
  }

  #[must_use]
  pub fn iter<'s>(&'s self) -> ParentedListIterator<'s, 'p, T> {
    ParentedListIterator {
      next: self.last.as_ref()
    }
  }
}

impl<'p, T> Default for ParentedList<'p, T> {
  fn default() -> Self {
    Self { last: None }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ParentNode<'p, T> {
  parent:    Option<&'p ParentNode<'p, T>>,
  pub value: T
}

#[derive(Debug, Clone, Copy)]
pub struct ParentedListIterator<'i, 'p, T> {
  next: Option<&'i ParentNode<'p, T>>
}

impl<'i, 'p, T> Iterator for ParentedListIterator<'i, 'p, T> {
  type Item = &'i T;

  fn next(&mut self) -> Option<Self::Item> {
    let Some(current) = self.next else {
      return None
    };
    self.next = current.parent;

    Some(&current.value)
  }
}

#[cfg(test)]
mod tests {

  use std::assert_matches::assert_matches;

  use super::*;

  #[test]
  fn not_mut() {
    let list: ParentedList<'_, i32> = Default::default();
    let _ = list.with_appended(1);
  }

  #[test]
  fn creates_new_node_with_value() {
    let list: ParentedList<'_, i32> = Default::default();
    let new_list = list.with_appended(1);
    assert_matches!(
      new_list.last,
      Some(ParentNode {
        parent: None,
        value:  1
      })
    )
  }

  #[test]
  fn iterates_empty() {
    let list: ParentedList<'_, i32> = Default::default();
    assert_matches!(list.iter().next(), None);
  }

  #[test]
  fn iterates_single_value() {
    let list: ParentedList<'_, i32> = Default::default();
    let list = list.with_appended(1);
    let mut iter = list.iter();
    assert_matches!(iter.next(), Some(1));
    assert_matches!(iter.next(), None);
  }

  #[test]
  fn iterates_multiple_values() {
    let list: ParentedList<'_, i32> = Default::default();
    let list = list.with_appended(1);
    let list = list.with_appended(2);
    let mut iter = list.iter();
    assert_matches!(iter.next(), Some(2));
    assert_matches!(iter.next(), Some(1));
    assert_matches!(iter.next(), None);
  }
}
