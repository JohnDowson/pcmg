use eframe::epaint::mutex::RwLockReadGuard;
use std::{
    collections::VecDeque,
    sync::Arc,
};

pub struct SampleQueue {
    inner: Arc<eframe::epaint::mutex::RwLock<VecDeque<f32>>>,
}

impl Clone for SampleQueue {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl SampleQueue {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn put(&self, sample: f32) {
        let mut g = self.inner.write();
        g.push_back(sample);
        if g.len() >= 44000 {
            g.pop_front();
        }
    }

    pub fn get(&self) -> RwLockReadGuard<VecDeque<f32>> {
        self.inner.read()
    }
}

impl Default for SampleQueue {
    fn default() -> Self {
        Self::new()
    }
}
