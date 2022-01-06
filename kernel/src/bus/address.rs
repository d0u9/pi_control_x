use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Address {
    inner: String,
}

impl Address {
    pub fn new(name: &str) -> Self {
        Self {
            inner: name.to_owned(),
        }
    }
}


impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
