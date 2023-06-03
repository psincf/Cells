use std::cell::UnsafeCell;
pub struct NoAlias<T>(UnsafeCell<T>);

impl<T> NoAlias<T> {
    pub fn new(data: T) -> NoAlias<T> {
        let data_cell = UnsafeCell::new(data);

        NoAlias(data_cell)
    }
}

impl<T> std::ops::Deref for NoAlias<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*(self.0.get()) }
    }
}

impl<T> std::ops::DerefMut for NoAlias<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.0.get()) }
    }
}