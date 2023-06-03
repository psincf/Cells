use std::cell::UnsafeCell;

pub struct StaticData<T> {
    data: UnsafeCell<T>
}

impl<T> StaticData<T> {
    pub const fn new(data: T) -> StaticData<T> {
        StaticData {
            data: UnsafeCell::new(data)
        }
    }

    pub fn get(&self) -> &'static T {
        unsafe { &*self.data.get() }
    }

    pub fn get_mut(&self) -> &'static mut T {        
        unsafe { &mut *self.data.get() }
    }

    pub fn set(&self, data: T) {
        unsafe { *self.data.get() = data };
    }
}

unsafe impl<T> std::marker::Sync for StaticData<T> {}

pub struct StaticDataPtr<T> {
    ptr: UnsafeCell<*mut T>
}

impl<T> StaticDataPtr<T> {
    pub const fn new() -> StaticDataPtr<T> {
        StaticDataPtr {
            ptr: UnsafeCell::new(std::ptr::null_mut())
        }
    }

    pub fn get(&self) -> &'static T {
        unsafe { &**self.ptr.get() }
    }

    pub fn get_mut(&self) -> &'static mut T {        
        unsafe { &mut **self.ptr.get() }
    }

    pub fn set(&self, ptr: *mut T) {
        unsafe { *self.ptr.get() = ptr };
    }
}

unsafe impl<T> std::marker::Sync for StaticDataPtr<T> {}

