use std::marker::{Send, Sync};
use std::fmt::Debug;
use std::future::Future;

use super::super::switch::*;

pub(super) struct DevicePoller;

pub trait Pollable {
    // fn pollable(self, shutdown: impl Future<Output = ()> + 'static) -> Box<dyn Future<Output = ()>>;
    fn pollable(self) -> Box<dyn Future<Output = ()>>;
    // fn pollable(self) {
}

impl<T> Pollable for Switch<T>
where
    T: Clone + Debug + Send + Sync + 'static
{
    // fn pollable(self, shutdown: impl Future<Output = ()> + 'static) -> Box<dyn Future<Output = ()>> {
    // fn pollable(self) {
    fn pollable(self) -> Box<dyn Future<Output = ()>> {
        Box::new(self.poll())
    }
}


