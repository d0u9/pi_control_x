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

    fn get_name(&self) -> String;
}

impl Debug for dyn SwitchDev {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Switch({})]", self.get_name())
    }
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

    fn get_name(&self) -> String {
        self.get_name()
    }
}

pub trait RouterDev: Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_poller(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    fn get_name(&self) -> String;
}

impl Debug for dyn RouterDev {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Router({})]", self.get_name())
    }
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

    fn get_name(&self) -> String {
        self.get_name().clone()
    }
}
