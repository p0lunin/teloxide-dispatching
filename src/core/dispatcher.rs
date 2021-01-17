use crate::core::{handler::Handler, service::Service};
use futures::{Stream, StreamExt};
use std::future::Future;
use std::pin::Pin;

type BoxFut<Upd> = Pin<Box<dyn Future<Output = Result<(), Upd>>>>;

pub struct Dispatcher<Upd> {
    handlers: Vec<Service<Upd>>,
}

impl<Upd: 'static> Dispatcher<Upd> {
    pub fn new() -> Self {
        Dispatcher {
            handlers: Vec::new(),
        }
    }

    pub fn service(mut self, service: impl Into<Service<Upd>>) -> Self {
        self.handlers.push(service.into());
        self
    }

    pub async fn dispatch_one(&self, upd: Upd) {
        self.handle(upd).await.unwrap_or_else(|_| panic!());
    }

    pub async fn dispatch_stream(&self, stream: impl Stream<Item = Upd>) {
        stream
            .for_each_concurrent(None, |upd| async move {
                self.dispatch_one(upd).await;
            })
            .await;
    }
}

impl<Upd: 'static> Handler<Upd, BoxFut<Upd>> for Dispatcher<Upd> {
    fn handle(&self, update: Upd) -> BoxFut<Upd> {
        let mut handler = None;
        for service in self.handlers.iter() {
            if service.guards.check(&update) {
                handler = Some(service.handler.clone());
                break;
            }
        }
        Box::pin(async move {
            match handler {
                Some(handler) => Ok(handler.handle(update).await),
                None => Err(update),
            }
        })
    }
}
