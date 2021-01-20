use crate::core::DispatchError;
use std::future::Future;

pub trait ErrorHandler<Upd, Err, Fut: Future<Output = ()>> {
    fn handle_error(&self, err: DispatchError<Upd, Err>) -> Fut;
}

impl<F, Upd, Err, Fut: Future<Output = ()>> ErrorHandler<Upd, Err, Fut> for F
where
    F: Fn(DispatchError<Upd, Err>) -> Fut,
{
    fn handle_error(&self, err: DispatchError<Upd, Err>) -> Fut {
        self(err)
    }
}
