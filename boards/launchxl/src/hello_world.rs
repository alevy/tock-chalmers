//! Sample capsule for Tock course. Prints 'Hello World' every second

use kernel::hil::time::{self, Alarm, Frequency};

pub struct HelloWorld<'a, A: Alarm + 'a>  {
    alarm: &'a A,
}

impl<'a, A: Alarm> HelloWorld<'a, A> {
    pub fn new(alarm: &'a A) -> HelloWorld<'a, A> {
        HelloWorld {
           alarm: alarm,
        }
    }

    pub fn start(&self) {
        debug!("Hello World");
    }
}

impl<'a, A: Alarm> time::Client for HelloWorld<'a, A> {
    fn fired(&self) {
    }
}

