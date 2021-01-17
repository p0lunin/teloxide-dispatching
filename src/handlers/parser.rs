use crate::core::{Guards, HandleFuture, Handler, HandlerInto, Service};
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct NotInitialized;
pub struct Initialized;

pub struct UpdateParser<GenUpd, Parser, Init>(Service<GenUpd>, PhantomData<(Parser, Init)>);

impl<GenUpd, Parser, Init> UpdateParser<GenUpd, Parser, Init> {
    pub fn into_inner(self) -> Service<GenUpd> {
        self.0
    }
}

impl<GenUpd, Parser> UpdateParser<GenUpd, Parser, NotInitialized>
where
    GenUpd: 'static,
    Parser: ParseUpdate<GenUpd> + 'static,
{
    pub fn new() -> Self {
        let service = Service::new(Guards::new().add(Parser::check), || async move {});
        UpdateParser(service, PhantomData)
    }

    pub fn to<F, H, Fut>(self, f: F) -> UpdateParser<GenUpd, Parser, Initialized>
    where
        H: Handler<Parser::Upd, Fut> + 'static,
        F: HandlerInto<H>,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let Service { guards, handler: _ } = self.0;
        let handler = f.into_handler();
        let service = Service {
            guards,
            handler: Arc::new(UpdateParserHandler::<_, Parser, GenUpd, _>::new(handler)),
        };
        UpdateParser(service, PhantomData)
    }
}

impl<GenUpd, Upd> Into<Service<GenUpd>> for UpdateParser<GenUpd, Upd, Initialized> {
    fn into(self) -> Service<GenUpd> {
        self.0
    }
}

pub trait ParseUpdate<GenUpd> {
    type Upd;

    fn check(update: &GenUpd) -> bool;
    fn parse(update: GenUpd) -> Self::Upd;
}

pub struct UpdateParserHandler<HandlerT, Parser, GenUpd, Fut> {
    handler: Arc<HandlerT>,
    phantom: PhantomData<(Parser, GenUpd, Fut)>,
}

impl<HandlerT, Parser, GenUpd, Fut> UpdateParserHandler<HandlerT, Parser, GenUpd, Fut>
where
    HandlerT: Handler<Parser::Upd, Fut>,
    Fut: Future<Output = ()>,
    Parser: ParseUpdate<GenUpd>,
{
    pub fn new(handler: HandlerT) -> Self {
        UpdateParserHandler {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

impl<HandlerT, Parser, GenUpd, Fut> Handler<GenUpd, HandleFuture<()>>
    for UpdateParserHandler<HandlerT, Parser, GenUpd, Fut>
where
    HandlerT: Handler<Parser::Upd, Fut>,
    Fut: Future<Output = ()> + Send + 'static,
    Parser: ParseUpdate<GenUpd>,
{
    fn handle(&self, data: GenUpd) -> HandleFuture<()> {
        let message = Parser::parse(data);
        let handler = self.handler.clone();
        Box::pin(handler.handle(message)) as _
    }
}
