use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, Copy)]
pub enum Primitives {
  Float,
  Int,
  String,
  Bool,
  Unknown
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
  None,
  Low,
  Medium,
  High
}

#[derive(Debug, Clone)]
pub enum ValueType {
  Struct {
    fields: Vec<Rc<RefCell<LinkedValueType>>>
  },
  Array {
    item_type: Rc<RefCell<LinkedValueType>>
  },
  Function {
    params:  Vec<LinkedValueType>,
    returns: Rc<RefCell<LinkedValueType>>
  },
  Primitive(Primitives),
  Ref(Rc<RefCell<LinkedValueType>>)
}

#[derive(Debug, Clone)]
pub struct ValueTypeInfo {
  pub ty:         ValueType,
  pub confidence: Confidence
}

#[derive(Debug, Clone)]
pub enum LinkedValueType {
  Type(ValueTypeInfo),
  Redirect(Rc<RefCell<LinkedValueType>>)
}

impl LinkedValueType {
  pub fn link(_a: &Rc<RefCell<LinkedValueType>>, _b: &Rc<RefCell<LinkedValueType>>) {
    // let a_concrete_ptr = Self::get_concrete_ptr(a);
    // let b_concrete_ptr = Self::get_concrete_ptr(b);
    // if Rc::ptr_eq(&a_concrete_ptr, &b_concrete_ptr) {
    //   return;
    // }

    // let a_concrete = a.borrow().get_concrete();
    // let b_concrete = b.borrow().get_concrete();

    // if a_concrete.confidence > b_concrete.confidence {
    //   *b_concrete_ptr.borrow_mut() = LinkedValueType::Redirect(a.clone())
    // } else {
    //   *a_concrete_ptr.borrow_mut() = LinkedValueType::Redirect(b.clone())
    // }
  }

  pub fn new_primitive(primitive: Primitives) -> Self {
    Self::Type(ValueTypeInfo {
      ty:         ValueType::Primitive(primitive),
      confidence: Confidence::None
    })
  }

  pub fn new_vector3() -> Self {
    Self::Type(ValueTypeInfo {
      ty:         ValueType::Struct {
        fields: vec![
          Self::new_primitive(Primitives::Float).make_shared(),
          Self::new_primitive(Primitives::Float).make_shared(),
          Self::new_primitive(Primitives::Float).make_shared(),
        ]
      },
      confidence: Confidence::High
    })
  }

  pub fn new_ref(ref_type: Rc<RefCell<LinkedValueType>>) -> Self {
    Self::Type(ValueTypeInfo {
      ty:         ValueType::Ref(ref_type),
      confidence: Confidence::None
    })
  }

  pub fn confidence(&mut self, confidence: Confidence) -> &mut Self {
    match self {
      LinkedValueType::Type(t) => {
        t.confidence = confidence;
      }
      LinkedValueType::Redirect(r) => {
        r.borrow_mut().confidence(confidence);
      }
    };
    self
  }

  pub fn make_shared(self) -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(self))
  }

  pub fn ref_type(&mut self) -> Rc<RefCell<Self>> {
    match self {
      LinkedValueType::Type(t) => {
        if let ValueTypeInfo {
          ty: ValueType::Ref(r),
          confidence
        } = t
        {
          *confidence = Confidence::High;
          r.clone()
        } else {
          let inner = Self::new_primitive(Primitives::Unknown).make_shared();
          *t = ValueTypeInfo {
            ty:         ValueType::Ref(inner.clone()),
            confidence: Confidence::High
          };
          inner
        }
      }
      LinkedValueType::Redirect(r) => r.borrow_mut().ref_type()
    }
  }

  pub fn struct_field(info: &Rc<RefCell<Self>>, field: usize) -> Rc<RefCell<Self>> {
    let borrowed: &mut Self = &mut info.borrow_mut();
    match borrowed {
      LinkedValueType::Type(t) => {
        if let ValueType::Struct { fields } = &mut t.ty {
          if fields.len() <= field {
            fields.resize_with(field + 1, || {
              Self::new_primitive(Primitives::Unknown).make_shared()
            });
          }
          fields[field].clone()
        } else if field > 0 {
          let mut fields = (0..field + 1)
            .map(|_| Self::new_primitive(Primitives::Unknown).make_shared())
            .collect::<Vec<_>>();
          fields[0] = LinkedValueType::Type(t.clone()).make_shared();
          let field = fields[field].clone();
          *t = ValueTypeInfo {
            ty:         ValueType::Struct { fields },
            confidence: Confidence::Medium
          };
          field
        } else {
          info.clone()
        }
      }
      LinkedValueType::Redirect(r) => Self::struct_field(r, field)
    }
  }

  pub fn array_item_type(&mut self) -> Rc<RefCell<Self>> {
    match self {
      LinkedValueType::Type(t) => {
        if let ValueType::Array { item_type } = &mut t.ty {
          item_type.clone()
        } else {
          let item_type = Self::new_primitive(Primitives::Unknown).make_shared();
          *t = ValueTypeInfo {
            ty:         ValueType::Array {
              item_type: item_type.clone()
            },
            confidence: Confidence::High
          };
          item_type
        }
      }
      LinkedValueType::Redirect(r) => r.borrow_mut().array_item_type()
    }
  }

  pub fn struct_size(&mut self, size: usize) {
    if size <= 1 {
      return;
    }

    match self {
      LinkedValueType::Type(t) => {
        if let ValueType::Struct { fields } = &mut t.ty {
          if fields.len() <= size {
            fields.resize_with(size, || {
              Self::new_primitive(Primitives::Unknown).make_shared()
            })
          } else {
            // TODO: func_605
            // panic!("Struct sized down???")
          }
        } else {
          *t = ValueTypeInfo {
            ty:         ValueType::Struct {
              fields: (0..size)
                .map(|_| Self::new_primitive(Primitives::Unknown).make_shared())
                .collect()
            },
            confidence: Confidence::Medium
          }
        }
      }
      LinkedValueType::Redirect(r) => r.borrow_mut().struct_size(size)
    }
  }

  pub fn hint(&mut self, ty: ValueTypeInfo) {
    match self {
      LinkedValueType::Type(t) => {
        match (&ty.ty, &t.ty) {
          (ValueType::Primitive(_), ValueType::Struct { fields }) => {
            let field = &fields[0];
            field.borrow_mut().hint(ty)
          }
          _ => {
            if ty.confidence > t.confidence {
              t.ty = ty.ty;
              t.confidence = ty.confidence;
            }
          }
        }
      }
      LinkedValueType::Redirect(r) => r.borrow_mut().hint(ty)
    }
  }

  pub fn size(&self) -> usize {
    match self {
      LinkedValueType::Type(t) => {
        match &t.ty {
          ValueType::Struct { fields } => fields.iter().map(|f| f.borrow().size()).sum(),
          ValueType::Array { .. } => 1,
          ValueType::Function { .. } => 1,
          ValueType::Primitive(_) => 1,
          ValueType::Ref(_) => 1
        }
      }
      LinkedValueType::Redirect(r) => r.borrow().size()
    }
  }

  pub fn get_concrete(&self) -> ValueTypeInfo {
    match self {
      LinkedValueType::Type(t) => t.clone(),
      LinkedValueType::Redirect(r) => r.borrow().get_concrete()
    }
  }

  // fn get_concrete_ptr(ty: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
  //   let rf: &Self = &ty.borrow();
  //   match rf {
  //     LinkedValueType::Type(_) => ty.clone(),
  //     LinkedValueType::Redirect(r) => Self::get_concrete_ptr(r)
  //   }
  // }
}
