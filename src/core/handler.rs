mod parser_handler;

pub use parser_handler::ParserHandler;

use crate::core::context::{Context, FromContext};
use futures::future::BoxFuture;
use std::future::Future;
use std::marker::PhantomData;

pub type HandleFuture<T> = BoxFuture<'static, T>;

pub trait Handler<Data, Fut: Future> {
    fn handle(&self, data: Data) -> Fut;
}

pub trait HandlerInto<T> {
    fn into_handler(self) -> T;
}

pub struct FnHandlerWrapper<F, P, Fut> {
    f: F,
    phantom: PhantomData<(P, Fut)>,
}

impl<F, P, Fut> FnHandlerWrapper<F, P, Fut> {
    pub fn new(f: F) -> Self {
        FnHandlerWrapper {
            f,
            phantom: PhantomData,
        }
    }
}

impl<Upd, F, Fut> Handler<Upd, HandleFuture<Fut::Output>> for FnHandlerWrapper<F, (), Fut>
where
    F: Fn() -> Fut,
    Fut: Future + Send + 'static,
{
    fn handle(&self, _: Upd) -> HandleFuture<Fut::Output> {
        Box::pin((self.f)()) as _
    }
}

impl<F, Upd, A, Fut> Handler<Upd, HandleFuture<Fut::Output>> for FnHandlerWrapper<F, (A,), Fut>
where
    A: FromContext<Upd>,
    F: Fn(A) -> Fut,
    Fut: Future + Send + 'static,
{
    fn handle(&self, update: Upd) -> HandleFuture<Fut::Output> {
        let context = Context::new(&update);
        Box::pin((self.f)(FromContext::from_context(&context))) as _
    }
}
/*
impl<F> HandlerInto<F> for F {
    fn into_handler(self) -> F {
        self
    }
}*/

impl<F, Fut: Future> HandlerInto<FnHandlerWrapper<F, (), Fut>> for F
where
    F: Fn() -> Fut,
{
    fn into_handler(self) -> FnHandlerWrapper<F, (), Fut> {
        FnHandlerWrapper::new(self)
    }
}

impl<F, A, Fut> HandlerInto<FnHandlerWrapper<F, (A,), Fut>> for F
where
    F: Fn(A) -> Fut,
{
    fn into_handler(self) -> FnHandlerWrapper<F, (A,), Fut> {
        FnHandlerWrapper::new(self)
    }
}
