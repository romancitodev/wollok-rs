use crate::value::Value;

/// One method activation.
pub struct Frame {
  pub receiver: Value,
  pub locals: Vec<Value>,
  pub stack: Vec<Value>,
  pub ip: usize,
}

impl Frame {
  #[must_use]
  pub fn new(receiver: Value, locals: Vec<Value>) -> Self {
    Self {
      receiver,
      locals,
      stack: Vec::new(),
      ip: 0,
    }
  }

  pub fn push(&mut self, value: Value) {
    self.stack.push(value);
  }

  /// # Panics
  /// On stack underflow.
  pub fn pop(&mut self) -> Value {
    self.stack.pop().expect("operand stack underflow")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn push_then_pop_round_trips() {
    let mut frame = Frame::new(Value::Null, vec![]);
    frame.push(Value::from(7i64));
    assert_eq!(frame.pop().as_int(), Some(7));
  }

  #[test]
  fn pops_in_lifo_order() {
    let mut frame = Frame::new(Value::Null, vec![]);
    frame.push(Value::from(1i64));
    frame.push(Value::from(2i64));
    assert_eq!(frame.pop().as_int(), Some(2));
    assert_eq!(frame.pop().as_int(), Some(1));
  }

  #[test]
  #[should_panic(expected = "operand stack underflow")]
  fn popping_an_empty_stack_panics() {
    let mut frame = Frame::new(Value::Null, vec![]);
    frame.pop();
  }

  #[test]
  fn locals_start_as_given_and_are_independently_indexable() {
    let frame = Frame::new(Value::Null, vec![Value::from(1i64), Value::from(2i64)]);
    assert_eq!(frame.locals[0].as_int(), Some(1));
    assert_eq!(frame.locals[1].as_int(), Some(2));
  }
}
