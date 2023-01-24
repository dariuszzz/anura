#[derive(Debug)]
pub enum IndigoError<T> {
    FatalError { msg: T },
}
