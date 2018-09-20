//! Sample capsule for Tock course at SOSP. It handles an alarm to
//! sample the ambient light sensor.

#![feature(const_fn, const_cell_new)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

use kernel::hil::time::{self, Alarm, Frequency};

pub struct HelloWorld<'a, A: Alarm + 'a> {
    alarm: &'a A,
}

impl<'a, A: Alarm> HelloWorld<'a, A> {
    pub fn new(alarm: &'a A) -> HelloWorld<'a, A> {
        HelloWorld {
            alarm: alarm,
        }
    }

    pub fn start(&self) {
        debug!("Hello Kernel");
    }
}

impl<'a, A: Alarm> time::Client for HelloWorld<'a, A> {
    fn fired(&self) {}
}
