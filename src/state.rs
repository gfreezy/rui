use std::fmt::Debug;

pub trait State: Debug {}
pub type AnyState = Box<dyn State + 'static>;
