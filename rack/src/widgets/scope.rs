use eframe::epaint::mutex::RwLockReadGuard;
use std::{
    collections::VecDeque,
    sync::Arc,
};

pub struct SampleQueue {
    period: usize,
    inner: Arc<eframe::epaint::mutex::RwLock<VecDeque<f32>>>,
}

impl Clone for SampleQueue {
    fn clone(&self) -> Self {
        Self {
            period: self.period,
            inner: Arc::clone(&self.inner),
        }
    }
}

impl SampleQueue {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            inner: Default::default(),
        }
    }

    pub fn put(&self, sample: f32) {
        let mut g = self.inner.write();
        g.push_back(sample);
        if g.len() >= self.period {
            g.pop_front();
        }
    }

    pub fn get(&self) -> RwLockReadGuard<VecDeque<f32>> {
        self.inner.read()
    }
}
