#[derive(Debug)]
pub enum IndigoError<T> {
    FatalError { msg: T },
}

impl<T> std::fmt::Display for IndigoError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Indigo Error").finish()
    }
}

impl<T: std::fmt::Display + std::fmt::Debug> std::error::Error for IndigoError<T> {}
