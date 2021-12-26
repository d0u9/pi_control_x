use ::std::collections::HashMap;

pub(super) struct Router<T, P> {
    inner: Option<T>,
    inner2: Option<P>,
}

impl<T, P> Router<T, P> {
    pub(super) fn new() -> Self {
        Self {
            inner: None,
            inner2: None,
        }
    }
}
