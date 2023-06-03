pub struct VecBool {
    len: usize,
    inner: Vec<u32>
}

impl VecBool {
    pub fn new() -> VecBool {
        VecBool {
            len: 0,
            inner: Vec::new(),
        }
    }

    pub fn push(&mut self, val: bool) {
        let len_inner = self.inner.len();
        if self.len == len_inner * 32 {
            self.inner.push(0);
        }

        let index_bool = self.len / 32;
        let index_bit = self.len % 32;
        let bitval = if val { 1 } else { 0 };

        let bitfield = bitval << (index_bit % 32) as u32;
        self.inner[index_bool] |= bitfield;
        self.len += 1;
    }

    #[inline(always)]
    pub fn last(&self) -> bool {
        self.get(self.len - 1)
    }

    #[inline(always)]
    pub fn get(&self, index: usize) -> bool {
        assert!(index < self.len);
        let index_bool = index / 32;
        let bitfield = 1u32 << (index % 32) as u32;
        (self.inner[index_bool] & bitfield) == bitfield
    }

    #[inline(always)]
    pub fn set(&mut self, index: usize, val: bool) {
        if val { self.set_true(index) } else { self.set_false(index) }
    }

    #[inline(always)]
    pub fn set_true(&mut self, index: usize) {
        assert!(index < self.len);
        let index_bool = index / 32;
        let bitfield = 0 ^ (1 << (index % 32) as u32);
        self.inner[index_bool] |= bitfield;
    }

    #[inline(always)]
    pub fn set_false(&mut self, index: usize) {
        assert!(index < self.len);
        let index_bool = index / 32;
        let bitfield = u32::MAX ^ (1 << (index % 32) as u32);
        self.inner[index_bool] &= bitfield;
    }

    pub fn swap_remove(&mut self, index: usize) {
        assert!(index < self.len);
        let index_bool = index / 32;
        let index_bit = self.len % 32;

        self.set(index, self.last());
        self.len -= 1;
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.len = 0;
    }
}