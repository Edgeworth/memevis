use eyre::{eyre, Result};

pub trait StringErrorConversion {
    type Value;

    fn serr(self) -> Result<Self::Value>;
}

impl<T, E: std::fmt::Debug> StringErrorConversion for Result<T, E> {
    type Value = T;

    fn serr(self) -> Result<T> {
        self.map_err(|e| eyre!("{:?}", e))
    }
}
