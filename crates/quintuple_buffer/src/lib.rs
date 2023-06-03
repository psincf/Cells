use std::ops::DerefMut;

use parking_lot::{Mutex, MutexGuard};

struct QuintupleBufferInnerInfo {
    pre_last_get: usize,
    last_get: usize,
    pre_last_set: usize,
    last_set: usize,
    last_set_ptr: usize,
}

pub struct QuintupleBuffer<T> {
    info: Mutex<QuintupleBufferInnerInfo>,
    buffers: [Mutex<T>; 5],
}

impl<T> QuintupleBuffer<T> {
    pub fn new(data: T) -> QuintupleBuffer<T> where T:Clone {
        QuintupleBuffer {
            info: Mutex::new(QuintupleBufferInnerInfo {
                pre_last_get: 0,
                last_get: 0,
                pre_last_set: 1,
                last_set: 2,
                last_set_ptr: 3,
            }),
            buffers: [
                Mutex::new(data.clone()),
                Mutex::new(data.clone()),
                Mutex::new(data.clone()),
                Mutex::new(data.clone()),
                Mutex::new(data.clone())
            ],
        }
    }

    pub fn get_last(&self) -> MutexGuard<T> {
        let mut info = self.info.lock();
        info.last_set;

        let buffer = &self.buffers[info.last_set];
        info.last_get = info.last_set;
        return buffer.lock()
    }
    

    pub fn get_pre_last(&self) -> MutexGuard<T> {
        let mut info = self.info.lock();
        info.pre_last_set;

        let buffer = &self.buffers[info.pre_last_set];
        info.pre_last_get = info.pre_last_set;
        return buffer.lock()
    }

    pub fn get_pre_last_and_last(&self) -> (MutexGuard<T>, MutexGuard<T>) {
        let mut info = self.info.lock();

        let buffer_last = &self.buffers[info.last_set];
        let buffer_pre_last = &self.buffers[info.pre_last_set];
        info.last_get = info.last_set;
        info.pre_last_get = info.pre_last_set;
        return (buffer_pre_last.lock(), buffer_last.lock())
    }

    pub fn set(&self, data: T) {
        let mut info = self.info.lock();
        let mut buffers_to_get = vec![0, 1, 2, 3, 4];

        buffers_to_get.retain( |&index| { index != info.last_get && index != info.pre_last_get && index != info.last_set && index != info.pre_last_set } );

        let buffer_index = *buffers_to_get.first().unwrap();
        let buffer = &self.buffers[buffer_index];
        *buffer.lock() = data;
        info.pre_last_set = info.last_set;
        info.last_set = buffer_index;
    }

    pub unsafe fn set_with_ptr(&self) -> &mut T {
        let mut info = self.info.lock();
        let mut buffers_to_get = vec![0, 1, 2, 3, 4];
        
        buffers_to_get.retain( |&index| { index != info.last_get && index != info.pre_last_get && index != info.last_set && index != info.pre_last_set } );
        
        let buffer_index = *buffers_to_get.first().unwrap();
        let buffer = &self.buffers[buffer_index];
        info.last_set_ptr = buffer_index;
        
        return &mut *(buffer.lock().deref_mut() as *mut T);
    }

    pub unsafe fn set_with_ptr_same(&self) -> &mut T {
        let info = self.info.lock();
        let buffer_index = info.last_set_ptr;
        let buffer = &self.buffers[buffer_index];

        return &mut *(buffer.lock().deref_mut() as *mut T);
    }

    pub unsafe fn change_ptr_last_set(&self) {
        let mut info = self.info.lock();
        info.pre_last_set = info.last_set;
        info.last_set = info.last_set_ptr;
    }
}