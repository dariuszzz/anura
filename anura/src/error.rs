#[derive(Debug)]
pub enum AnuraError<T> {
    FatalError { msg: T },
}

impl<T> std::fmt::Display for AnuraError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Anura Error").finish()
    }
}

impl<T: std::fmt::Display + std::fmt::Debug> std::error::Error for AnuraError<T> {}
