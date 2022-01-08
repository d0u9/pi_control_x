use std::marker::{Send, Sync};
use std::fmt::Debug;
use std::future::Future;

use async_trait::async_trait;

use super::super::switch::*;

pub(super) struct DevicePoller;

pub trait Pollable {
    fn pollable(self, shutdown: impl Future<Output = ()> + 'static) -> Box<dyn Future<Output = ()>>;
}

impl Pollable for Switch<T>
// where
//     T: Clone + Debug + Send + Sync + 'static
{
    fn pollable(self, shutdown: impl Future<Output = ()> + 'static) -> Box<dyn Future<Output = ()>> {
        Box::new(self.poll(shutdown))
    }
}


