use std::{
    alloc::{alloc, dealloc, Layout},
    marker::PhantomData,
    ptr::{self, Pointee},
};

pub struct Fused<Dyn: ?Sized> {
    inner: *mut u8,
    max_align: usize,
    len_items: usize,
    len_bytes: usize,
    cap_bytes: usize,
    _tag: PhantomData<Dyn>,
}

unsafe impl<Dyn: ?Sized> Send for Fused<Dyn> {}

impl<Dyn: ?Sized> Fused<Dyn> {
    pub fn new() -> Self {
        Self {
            inner: std::ptr::null_mut(),
            max_align: 0,
            len_items: 0,
            len_bytes: 0,
            cap_bytes: 0,
            _tag: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.len_items
    }

    fn realloc(&mut self, min_size: usize) {
        let old = self.inner;
        let old_layout =
            unsafe { Layout::from_size_align_unchecked(self.cap_bytes, self.max_align) };
        let size = if self.cap_bytes == 0 {
            min_size
        } else {
            self.cap_bytes
                .checked_mul(2)
                .and_then(|s| s.checked_add(min_size))
                .expect("New capacity overflowed usize")
        };
        unsafe {
            let layout = Layout::from_size_align_unchecked(size, self.max_align);
            self.cap_bytes = layout.pad_to_align().size();
            let new = alloc(layout);
            std::ptr::copy(old, new, self.len_bytes);
            self.inner = new;
            dealloc(old, old_layout);
        }
    }

    pub fn push<T: 'static>(&mut self, v: T, meta: <Dyn as Pointee>::Metadata) {
        let layout = Layout::new::<Unbox<T, Dyn>>();
        if self.max_align < layout.align() {
            self.max_align = layout.align()
        }
        if self.cap_bytes - self.len_bytes < layout.size() {
            self.realloc(layout.size())
        }
        let v = Unbox {
            size: layout.size(),
            meta,
            inner: v,
        };
        let last_size = self.get_size(self.len_items.saturating_sub(1));
        *last_size = round_up(*last_size, layout.align());
        self.len_bytes = round_up(self.len_bytes, layout.align());
        unsafe {
            self.inner
                .add(self.len_bytes)
                .cast::<Unbox<T, Dyn>>()
                .write(v)
        }
        self.len_bytes += layout.size();
        self.len_items += 1;
    }

    fn get_size(&mut self, n: usize) -> &mut usize {
        assert!(
            n <= self.len_items,
            "Assertion failed ({n}<{})",
            self.len_items
        );

        let mut item = self.inner;
        for _ in 0..n {
            unsafe {
                let size = (&*item.cast::<Unbox<(), Dyn>>()).size;
                item = item.add(size)
            }
        }
        unsafe { &mut (*item.cast::<Unbox<(), Dyn>>()).size }
    }

    pub fn get_dyn(&mut self, n: usize) -> &mut Dyn {
        assert!(n <= self.len_items);
        let mut item = self.inner;
        for _ in 0..n {
            unsafe {
                let size = (&*item.cast::<Unbox<(), Dyn>>()).size;
                item = item.add(size)
            }
        }
        unsafe {
            let item = &mut *item.cast::<Unbox<(), Dyn>>();
            let meta = item.meta;
            let ptr = &mut item.inner;
            &mut *(ptr::from_raw_parts::<Dyn>(ptr, meta) as *mut _)
        }
    }
}

#[repr(C)]
pub struct Unbox<T, Dyn: ?Sized> {
    size: usize,
    meta: <Dyn as Pointee>::Metadata,
    inner: T,
}

fn round_up(n: usize, m: usize) -> usize {
    if m == 0 {
        n
    } else {
        let rem = n % m;
        if rem == 0 {
            n
        } else {
            n + m - rem
        }
    }
}
