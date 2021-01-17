use crate::core::{handler::Handler, Guards, HandleFuture, HandlerInto};
use std::sync::Arc;

pub struct Service<Upd> {
    pub guards: Guards<Upd>,
    pub handler: Arc<dyn Handler<Upd, HandleFuture<()>>>,
}

impl<Upd> Service<Upd> {
    pub fn new<H, F>(guards: Guards<Upd>, handler: F) -> Self
    where
        H: Handler<Upd, HandleFuture<()>> + 'static,
        F: HandlerInto<H>,
    {
        Service {
            guards,
            handler: Arc::new(handler.into_handler()) as _,
        }
    }
}
