use std::convert::From;
use std::fmt::Debug;

use super::wire::Endpoint;

#[derive(Debug)]
pub struct Router<U, V> {
    endpoints: (Endpoint<U>, Endpoint<V>),
}

impl<U, V> Router<U, V>
where
    U: Clone + Debug + From<V>,
    V: Clone + Debug + From<U>,
{
    pub fn new(ep0: Endpoint<U>, ep1: Endpoint<V>) -> Self {
        Self {
            endpoints: (ep0, ep1),
        }
    }
}
