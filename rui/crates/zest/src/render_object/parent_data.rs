use std::{
    any::Any,
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub struct ParentData {
    inner: Rc<RefCell<dyn Any + 'static>>,
}

impl ParentData {
    pub fn new<T: Any + 'static>(data: T) -> Self {
        ParentData {
            inner: Rc::new(RefCell::new(data)) as Rc<RefCell<dyn Any>>,
        }
    }

    pub fn with<T: 'static, R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&*self.inner.borrow_mut().downcast_ref().unwrap())
    }

    pub fn with_mut<T: 'static, R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        f(&mut *self.inner.borrow_mut().downcast_mut::<T>().unwrap())
    }
}

#[derive(Clone)]
struct WeakParentData {
    inner: Weak<RefCell<dyn Any + 'static>>,
}
