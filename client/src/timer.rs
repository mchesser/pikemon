// FIXME: Swap to http://github.com/rust-lang/time crate when it works for me
#![allow(deprecated)]
#![allow(dead_code)]
extern crate time;
use self::time::precise_time_s;

pub struct Timer {
    last: f64
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            last: precise_time_s()
        }
    }

    pub fn elapsed(&self) -> f64 {
        precise_time_s() - self.last
    }

    pub fn elapsed_seconds(&self) -> f64 {
        self.elapsed()
    }

    pub fn reset(&mut self) {
        self.last = precise_time_s();
    }
}
