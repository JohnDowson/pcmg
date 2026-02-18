use eframe::epaint::mutex::RwLockReadGuard;
use std::{
    collections::VecDeque,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering,
        },
        Arc,
    },
};

pub struct SampleQueue {
    inner: Arc<(AtomicUsize, eframe::epaint::mutex::RwLock<VecDeque<f32>>)>,
}

impl Clone for SampleQueue {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl SampleQueue {
    pub fn new(period: usize) -> Self {
        Self {
            inner: Arc::new((period.into(), Default::default())),
        }
    }

    pub fn set_period(&self, period: usize) {
        self.inner.0.store(period, Ordering::Relaxed)
    }

    pub fn put(&self, sample: f32) {
        let mut g = self.inner.1.write();
        g.push_back(sample);
        if g.len() >= self.inner.0.load(Ordering::Relaxed) {
            g.pop_front();
        }
    }

    pub fn get(&'_ self) -> RwLockReadGuard<'_, VecDeque<f32>> {
        self.inner.1.read()
    }
}
