use crate::core::demux::DemuxBuilder;
use crate::core::dispatch_error::HandleResult;
use crate::core::error_handler::ErrorHandler;
use crate::core::{Demux, DispatchError, HandleFuture, Handler};
use futures::{Stream, StreamExt};
use std::future::Future;
use std::marker::PhantomData;

pub struct Dispatcher<Upd, Err, ErrHandler, HandlerFut> {
    demux: Demux<Upd, Err>,
    error_handler: ErrHandler,
    phantom: PhantomData<HandlerFut>,
}

impl<Upd, Err, ErrHandler, HandlerFut> Dispatcher<Upd, Err, ErrHandler, HandlerFut>
where
    Upd: 'static,
    ErrHandler: ErrorHandler<Upd, Err, HandlerFut>,
    HandlerFut: Future<Output = ()>,
{
    pub async fn dispatch_one(&self, upd: Upd) {
        match self.demux.handle(upd) {
            Ok(fut) => {
                let res = fut.await;
                match res {
                    HandleResult::Ok => {}
                    HandleResult::Err(e) => {
                        self.error_handler
                            .handle_error(DispatchError::HandlerError(e))
                            .await
                    }
                }
            }
            Err(e) => {
                self.error_handler
                    .handle_error(DispatchError::NoHandler(e))
                    .await
            }
        }
    }

    pub async fn dispatch_stream(&self, stream: impl Stream<Item = Upd>) {
        stream
            .for_each_concurrent(None, |upd| async move {
                self.dispatch_one(upd).await;
            })
            .await;
    }
}

pub struct DispatcherBuilder<Upd, Err, Handler, HandlerFut> {
    demux: DemuxBuilder<Upd, Err>,
    error_handler: Handler,
    phantom: PhantomData<HandlerFut>,
}

impl<Upd, Err> DispatcherBuilder<Upd, Err, (), ()> {
    pub fn new() -> Self {
        DispatcherBuilder {
            demux: DemuxBuilder::new(),
            error_handler: (),
            phantom: PhantomData,
        }
    }

    pub fn error_handler<H, Fut>(self, error_handler: H) -> DispatcherBuilder<Upd, Err, H, Fut>
    where
        H: ErrorHandler<Upd, Err, Fut>,
        Fut: Future<Output = ()>,
    {
        let DispatcherBuilder { demux, .. } = self;
        DispatcherBuilder {
            demux,
            error_handler,
            phantom: PhantomData,
        }
    }
}

impl<Upd, Err, ErrHandler, Fut> DispatcherBuilder<Upd, Err, ErrHandler, Fut> {
    pub fn handle(mut self, handler: impl Handler<Upd, Err, HandleFuture<Err>> + 'static) -> Self {
        self.demux.add_service(handler);
        self
    }
}

impl<Upd, Err, ErrHandler, Fut> DispatcherBuilder<Upd, Err, ErrHandler, Fut>
where
    ErrHandler: ErrorHandler<Upd, Err, Fut>,
    Fut: Future<Output = ()>,
{
    pub fn build(self) -> Dispatcher<Upd, Err, ErrHandler, Fut> {
        let DispatcherBuilder {
            demux,
            error_handler,
            ..
        } = self;
        Dispatcher {
            demux: demux.build(),
            error_handler,
            phantom: PhantomData,
        }
    }
}
