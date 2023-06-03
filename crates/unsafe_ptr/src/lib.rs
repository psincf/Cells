pub struct UnsafePtr<T: ?Sized>(*mut T);

impl<T: ?Sized> UnsafePtr<T> {
    pub fn new(ptr: *mut T) -> UnsafePtr<T> {
        UnsafePtr(ptr)
    }

    #[inline(always)]
    pub fn raw(&self) -> *mut T {
        self.0
    }

    #[inline(always)]
    pub unsafe fn ref_const(&self) -> &T {
        & *self.0
    }

    #[inline(always)]
    pub unsafe fn ref_mut(&self) -> &mut T {
        &mut *self.0
    }
}

impl<T: ?Sized> Clone for UnsafePtr<T> {
    fn clone(&self) -> UnsafePtr<T> {
        UnsafePtr(self.0)
    }
}

impl<T: ?Sized> Copy for UnsafePtr<T> {}

unsafe impl<T: ?Sized> Send for UnsafePtr<T> {}
unsafe impl<T: ?Sized> Sync for UnsafePtr<T> {}