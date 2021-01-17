use crate::core::handler::Handler;
use crate::core::{HandleFuture, HandlerInto};
use std::marker::PhantomData;
use std::pin::Pin;
use tokio::macros::support::Future;

pub struct ParserHandler<ParserT, Upd, NextUpd, HandlerT, HandlerFut> {
    parser: ParserT,
    handler: HandlerT,
    phantom: PhantomData<(Upd, NextUpd, HandlerFut)>,
}

impl<ParserT, Upd, NextUpd, HandlerT, HandlerFut>
    ParserHandler<ParserT, Upd, NextUpd, HandlerT, HandlerFut>
where
    ParserT: Parser<Upd, NextUpd>,
    HandlerT: Handler<NextUpd, HandlerFut>,
    HandlerFut: Future<Output = ()>,
{
    pub fn new<H>(parser: ParserT, handler: H) -> Self
    where
        H: HandlerInto<HandlerT>,
    {
        ParserHandler {
            parser,
            handler: handler.into_handler(),
            phantom: PhantomData,
        }
    }
}

impl<ParserT, Upd, NextUpd, HandlerT, HandlerFut> Handler<Upd, HandleFuture<()>>
    for ParserHandler<ParserT, Upd, NextUpd, HandlerT, HandlerFut>
where
    ParserT: Parser<Upd, NextUpd>,
    HandlerT: Handler<NextUpd, HandlerFut>,
    HandlerFut: Future<Output = ()> + Send + 'static,
{
    fn handle(&self, data: Upd) -> HandleFuture<()> {
        let parsed = self.parser.parse(data);
        Box::pin(self.handler.handle(parsed))
    }
}

pub trait Parser<From, To> {
    fn parse(&self, from: From) -> To;
}

impl<F, From, To> Parser<From, To> for F
where
    F: Fn(From) -> To,
{
    fn parse(&self, from: From) -> To {
        self(from)
    }
}
