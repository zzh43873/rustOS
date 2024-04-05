use core::{future::{self, Future}, pin::Pin};
use alloc::boxed::Box;

pub struct Task {
    future : Pin<Box<dyn future<output = ()>>>,
}

impl Task {
    pub fn new(future : impl future<output = ()> + 'static) -> Task {
        Task {
            future : Box::pin(future),
        }
    }
}

















