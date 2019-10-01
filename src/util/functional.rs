pub trait TransformingAndThen<T, E> {
    fn and_then_fwd<U, E2, F: FnOnce(T) -> Result<U, E2>>(self, op: F) -> Result<U, E>
        where E: std::convert::From<E2>;
}

impl<T, E> TransformingAndThen<T, E> for Result<T, E> {
    fn and_then_fwd<U, E2, F: FnOnce(T) -> Result<U, E2>>(self, op: F) -> Result<U, E>
        where E: std::convert::From<E2> {
        match self {
            Ok(t) => op(t).map_err(From::from),
            Err(e) => Err(e)
        }
    }
}
