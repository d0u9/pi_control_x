use std::pin::Pin;
use std::any::Any;
use std::fmt::Debug;
use std::future::Future;

use super::super::switch::*;

pub trait SwitchDev: Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_poller(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()>>>;
}

impl<T> SwitchDev for Switch<T> 
where
    T: 'static + Debug + Clone
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_poller(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(self.poll())
    }
}
