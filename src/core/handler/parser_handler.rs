use crate::core::dispatch_error::HandleResult;
use crate::core::handler::Handler;
use crate::core::{HandleFuture, IntoHandler};
use futures::FutureExt;
use std::marker::PhantomData;
use tokio::macros::support::Future;

pub struct ParserHandler<ParserT, Upd, NextUpd, Rest, Err, HandlerT, HandlerFut> {
    parser: ParserT,
    handler: HandlerT,
    phantom: PhantomData<(Upd, NextUpd, Rest, Err, HandlerFut)>,
}

impl<ParserT, Upd, NextUpd, Rest, Err, HandlerT, HandlerFut>
    ParserHandler<ParserT, Upd, NextUpd, Rest, Err, HandlerT, HandlerFut>
where
    ParserT: Parser<Upd, NextUpd, Rest>,
    HandlerT: Handler<NextUpd, Err, HandlerFut>,
    HandlerFut: Future,
    HandlerFut::Output: Into<HandleResult<Err>>,
{
    pub fn new<H>(parser: ParserT, handler: H) -> Self
    where
        H: IntoHandler<HandlerT>,
    {
        ParserHandler {
            parser,
            handler: handler.into_handler(),
            phantom: PhantomData,
        }
    }
}

impl<ParserT, Upd, Err, NextUpd, Rest, HandlerT, HandlerFut> Handler<Upd, Err, HandleFuture<Err>>
    for ParserHandler<ParserT, Upd, NextUpd, Rest, Err, HandlerT, HandlerFut>
where
    Err: 'static,
    ParserT: Parser<Upd, NextUpd, Rest>,
    Upd: RecombineFrom<ParserT, From = NextUpd, Rest = Rest>,
    HandlerT: Handler<NextUpd, Err, HandlerFut>,
    HandlerFut: Future + Send + 'static,
    HandlerFut::Output: Into<HandleResult<Err>>,
{
    fn handle(&self, data: Upd) -> Result<HandleFuture<Err>, Upd> {
        match self.parser.parse(data) {
            Ok(ParserOut { data: next, rest }) => match self.handler.handle(next) {
                Ok(fut) => Ok(Box::pin(fut.map(Into::into)) as _),
                Err(next) => {
                    let upd = Upd::recombine(ParserOut::new(next, rest));
                    Err(upd)
                }
            },
            Err(upd) => Err(upd),
        }
    }
}

pub struct ParserOut<T, Rest> {
    pub data: T,
    pub rest: Rest,
}

impl<T, Rest> ParserOut<T, Rest> {
    pub fn new(data: T, rest: Rest) -> Self {
        ParserOut { data, rest }
    }

    pub fn into_inner(self) -> (T, Rest) {
        (self.data, self.rest)
    }
}

pub trait Parser<From, To, Rest> {
    fn parse(&self, from: From) -> Result<ParserOut<To, Rest>, From>;
}

impl<F, From, To, Rest> Parser<From, To, Rest> for F
where
    F: Fn(From) -> Result<ParserOut<To, Rest>, From>,
    From: RecombineFrom<F, From = To, Rest = Rest>,
{
    fn parse(&self, from: From) -> Result<ParserOut<To, Rest>, From> {
        self(from)
    }
}

pub trait RecombineFrom<Parser> {
    type From;
    type Rest;

    fn recombine(info: ParserOut<Self::From, Self::Rest>) -> Self;
}

pub struct MapParser<Parser1, Parser2, Parser1Out, Rest1, Rest2, Out>(
    Parser1,
    Parser2,
    PhantomData<(Parser1Out, Rest1, Rest2, Out)>,
);

impl<Parser1, Parser2, Parser1Out, Rest1, Rest2, Out>
    MapParser<Parser1, Parser2, Parser1Out, Rest1, Rest2, Out>
{
    pub fn new(field0: Parser1, field1: Parser2) -> Self {
        MapParser(field0, field1, PhantomData)
    }
}

impl<From, Intermediate, To, Parser1, Parser2, Rest1, Rest2, Out> Parser<From, To, (Rest1, Rest2)>
    for MapParser<Parser1, Parser2, Intermediate, Rest1, Rest2, Out>
where
    Parser1: Parser<From, Intermediate, Rest1>,
    Parser2: Parser<Intermediate, To, Rest2>,
    From: RecombineFrom<Parser1, From = Intermediate, Rest = Rest1>,
{
    fn parse(&self, from: From) -> Result<ParserOut<To, (Rest1, Rest2)>, From> {
        self.0.parse(from).and_then(
            |ParserOut {
                 data: intermediate,
                 rest: rest1,
             }| {
                match self.1.parse(intermediate) {
                    Ok(ParserOut {
                        data: res,
                        rest: rest2,
                    }) => Ok(ParserOut::new(res, (rest1, rest2))),
                    Err(ir) => Err(From::recombine(ParserOut::new(ir, rest1))),
                }
            },
        )
    }
}
/*
FIXME: overflow evaluating the requirement `Upd: RecombineFrom<MapParser<_, _, _, _, _, _>>
impl<Parser1, Parser2, Intermediate, Rest1, Rest2, Out, Origin> RecombineFrom<MapParser<Parser1, Parser2, Intermediate, Rest1, Rest2, Out>> for Origin
where
    Intermediate: RecombineFrom<Parser2, From = Out, Rest = Rest2>,
    Origin: RecombineFrom<Parser1, From = Intermediate, Rest = Rest1>,
{
    type From = Out;
    type Rest = (Rest1, Rest2);

    fn recombine(info: ParserOut<Self::From, Self::Rest>) -> Self {
        let (out, (rest1, rest2)) = info.into_inner();
        let ir = Intermediate::recombine(ParserOut::new(out, rest1));
        Origin::recombine(ParserOut::new(ir, rest2))
    }
}
*/
