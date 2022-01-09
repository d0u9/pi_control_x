use std::convert::From;
use std::pin::Pin;
use std::any::Any;
use std::fmt::Debug;
use std::future::Future;

use super::super::switch::*;
use super::super::router::*;

pub trait SwitchDev: Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_poller(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

impl<T> SwitchDev for Switch<T>
where
    T: 'static + Debug + Clone + Send
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_poller(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(self.poll())
    }
}

pub trait RouterDev: Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_poller(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

impl<U, V> RouterDev for Router<U, V>
where
    U: 'static + Debug + Clone + Send + From<V>,
    V: 'static + Debug + Clone + Send + From<U>,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_poller(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(self.poll())
    }
}
