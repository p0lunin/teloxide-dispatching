mod impls {
    use crate::core::{
        Demux, DemuxBuilder, Guard, Guards, HandleFuture, HandleResult, Handler, IntoHandler,
        MapParser, OrGuard, Parser, ParserOut, RecombineFrom,
    };
    use crate::handlers::parser::UpdateParser;
    use crate::updates::UpdateRest;
    use futures::FutureExt;
    use std::future::Future;
    use std::marker::PhantomData;
    use teloxide_core::types;
    use teloxide_core::types::{Message, Update};

    pub(crate) mod parser {
        pub struct Common;
        pub struct NewChatMembers;
        pub struct LeftChatMember;
        pub struct NewChatTitle;
        pub struct NewChatPhoto;
        pub struct DeleteChatPhoto;
        pub struct GroupChatCreated;
        pub struct SupergroupChatCreated;
        pub struct ChannelChatCreated;
        pub struct Migrate;
        pub struct Pinned;
        pub struct Invoice;
        pub struct SuccessfulPayment;
        pub struct ConnectedWebsite;
        pub struct PassportData;
        pub struct Dice;
    }

    macro_rules! impl_parser {
        ($($ty:ident,)*) => {
            $(
                impl Parser<Message, Message, ()> for parser::$ty {
                    fn parse(&self, update: Message) -> Result<ParserOut<Message, ()>, Message> {
                        match &update.kind {
                            teloxide_core::types::MessageKind::$ty(_) => Ok(ParserOut::new(update, ())),
                            _ => Err(update),
                        }
                    }
                }
            )*
        }
    }

    impl_parser!(
        Common,
        NewChatMembers,
        LeftChatMember,
        NewChatTitle,
        NewChatPhoto,
        DeleteChatPhoto,
        GroupChatCreated,
        SupergroupChatCreated,
        ChannelChatCreated,
        Migrate,
        Pinned,
        Invoice,
        SuccessfulPayment,
        ConnectedWebsite,
        PassportData,
        Dice,
    );
    impl<Parser1, Parser2>
        RecombineFrom<MapParser<Parser1, Parser2, Message, UpdateRest, (), Message>> for Update
    where
        Update: RecombineFrom<Parser1, From = Message, Rest = UpdateRest>,
    {
        type From = Message;
        type Rest = (UpdateRest, ());

        fn recombine(info: ParserOut<Self::From, Self::Rest>) -> Self {
            let (out, (rest1, _)) = info.into_inner();
            <Update as RecombineFrom<Parser1>>::recombine(ParserOut::new(out, rest1))
        }
    }

    struct GuardsHandler {
        guards: Guards<Message>,
    }

    impl GuardsHandler {
        pub fn new(guards: Guards<Message>) -> Self {
            GuardsHandler { guards }
        }
    }

    impl<Err> Handler<Message, Err, HandleFuture<Err>> for GuardsHandler {
        fn handle(&self, data: Message) -> Result<HandleFuture<Err>, Message> {
            match self.guards.check(&data) {
                true => Err(data),
                false => Ok(Box::pin(async { HandleResult::Ok })),
            }
        }
    }

    struct GuardHandler<Guard, Handler, Err, HFut> {
        guard: Guard,
        wrong_handler: Handler,
        phantom: PhantomData<(Err, HFut)>,
    }

    impl<Guard, Handler, Err, HFut> GuardHandler<Guard, Handler, Err, HFut> {
        pub fn new(guard: Guard, wrong_handler: Handler) -> Self {
            GuardHandler {
                guard,
                wrong_handler,
                phantom: PhantomData,
            }
        }
    }

    impl<GuardT, HandlerT, Err, HFut> Handler<Message, Err, HandleFuture<Err>>
        for GuardHandler<GuardT, HandlerT, Err, HFut>
    where
        GuardT: Guard<Message>,
        HandlerT: Handler<Message, Err, HFut>,
        HFut: Future + Send + 'static,
        HFut::Output: Into<HandleResult<Err>> + 'static,
        Err: 'static,
    {
        fn handle(&self, data: Message) -> Result<HandleFuture<Err>, Message> {
            match self.guard.check(&data) {
                true => Err(data),
                false => self
                    .wrong_handler
                    .handle(data)
                    .map(|fut| Box::pin(fut.map(Into::into)) as _),
            }
        }
    }

    pub struct MessageParser<UpdateParser, ParserT, Err> {
        update_parser: UpdateParser,
        parser: ParserT,
        demux: DemuxBuilder<Message, Err>,
        guards: Guards<Message>,
        last_guard: Option<Box<dyn Guard<Message>>>,
    }

    impl<UpdateParser, ParserT, Err> MessageParser<UpdateParser, ParserT, Err>
    where
        UpdateParser: Parser<Update, Message, UpdateRest>,
        ParserT: Parser<Message, Message, ()> + 'static,
        Update: RecombineFrom<UpdateParser, From = Message, Rest = UpdateRest>,
    {
        pub fn new(update_parser: UpdateParser, parser: ParserT) -> Self {
            MessageParser {
                update_parser,
                parser,
                demux: DemuxBuilder::new(),
                guards: Guards::new(),
                last_guard: None,
            }
        }
    }

    impl<UpdateParser, ParserT, Err> MessageParser<UpdateParser, ParserT, Err>
    where
        UpdateParser: Parser<Update, Message, UpdateRest>,
        ParserT: Parser<Message, Message, ()> + 'static,
        Update: RecombineFrom<UpdateParser, From = Message, Rest = UpdateRest>,
    {
        pub fn by<F, H, Fut>(
            self,
            f: F,
        ) -> MessageHandler<
            MapParser<UpdateParser, ParserT, Message, UpdateRest, (), Message>,
            H,
            Err,
        >
        where
            H: Handler<Message, Err, Fut> + 'static,
            F: IntoHandler<H>,
            Fut: Future + Send + 'static,
            Fut::Output: Into<HandleResult<Err>>,
        {
            let MessageParser {
                update_parser: parent,
                parser,
                demux,
                ..
            } = self;
            let parser = MapParser::new(parent, parser);
            MessageHandler {
                parser,
                handler: f.into_handler(),
                demux: demux.build(),
                phantom: PhantomData,
            }
        }
    }

    impl<UpdateParser, ParserT, Err> MessageParser<UpdateParser, ParserT, Err> {
        pub fn with_guard(mut self, guard: impl Guard<Message> + 'static) -> Self {
            let prev = self.last_guard.take();
            if let Some(prev) = prev {
                self.guards.add_boxed_guard(prev);
            }
            self.last_guard = Some(Box::new(guard) as _);
            self
        }

        pub fn or(mut self, guard: impl Guard<Message> + 'static) -> Self {
            let prev = self
                .last_guard
                .take()
                .expect("or function must be called after using .with_* funtion!");
            self.last_guard = Some(Box::new(OrGuard::new(prev, guard)) as _);
            self
        }

        pub fn or_else<F, H, HFut>(mut self, func: F) -> Self
        where
            F: IntoHandler<H>,
            H: Handler<Message, Err, HFut> + 'static,
            HFut: Future + Send + 'static,
            HFut::Output: Into<HandleResult<Err>> + 'static,
            Err: 'static,
        {
            let prev_guard = self
                .last_guard
                .take()
                .expect("or_else function must be called after using .with_* funtion!");
            let wrong_handler = func.into_handler();

            self.create_guards_service();
            self.demux
                .add_service(GuardHandler::new(prev_guard, wrong_handler));

            self
        }

        fn create_guards_service(&mut self) {
            if !self.guards.is_empty() {
                let mut guards = Guards::new();
                std::mem::swap(&mut guards, &mut self.guards);
                self.demux.add_service(GuardsHandler::new(guards));
            }
        }
    }

    impl<UpdateParser, ParserT, Err> MessageParser<UpdateParser, ParserT, Err> {
        pub fn with_id(self, guard: impl Guard<i32> + 'static) -> Self {
            self.with_guard(move |message: &Message| guard.check(&message.id))
        }

        pub fn with_date(self, guard: impl Guard<i32> + 'static) -> Self {
            self.with_guard(move |message: &Message| guard.check(&message.date))
        }

        pub fn with_chat(self, guard: impl Guard<types::Chat> + 'static) -> Self {
            self.with_guard(move |message: &Message| guard.check(&message.chat))
        }

        pub fn with_chat_id(self, guard: impl Guard<i64> + 'static) -> Self {
            self.with_guard(move |message: &Message| guard.check(&message.chat.id))
        }

        pub fn with_via_bot(self, guard: impl Guard<types::User> + 'static) -> Self {
            self.with_guard(move |message: &Message| match &message.via_bot {
                Some(bot) => guard.check(bot),
                None => false,
            })
        }

        pub fn with_from(self, guard: impl Guard<types::User> + 'static) -> Self {
            self.with_guard(move |message: &Message| match message.from() {
                Some(user) => guard.check(user),
                None => false,
            })
        }

        pub fn with_forward_from(self, guard: impl Guard<types::ForwardedFrom> + 'static) -> Self {
            self.with_guard(move |message: &Message| match message.forward_from() {
                Some(user) => guard.check(user),
                None => false,
            })
        }

        pub fn with_forward_from_chat(self, guard: impl Guard<types::Chat> + 'static) -> Self {
            self.with_guard(move |message: &Message| match message.forward_from_chat() {
                Some(chat) => guard.check(chat),
                None => false,
            })
        }

        pub fn with_forward_from_message_id(self, guard: impl Guard<i32> + 'static) -> Self {
            self.with_guard(
                move |message: &Message| match message.forward_from_message_id() {
                    Some(chat) => guard.check(chat),
                    None => false,
                },
            )
        }

        pub fn with_forward_signature(self, guard: impl Guard<str> + 'static) -> Self {
            self.with_guard(move |message: &Message| match message.forward_signature() {
                Some(chat) => guard.check(chat),
                None => false,
            })
        }

        pub fn with_forward_date(self, guard: impl Guard<i32> + 'static) -> Self {
            self.with_guard(move |message: &Message| match message.forward_date() {
                Some(chat) => guard.check(chat),
                None => false,
            })
        }

        pub fn with_text(self, guard: impl Guard<str> + 'static) -> Self {
            self.with_guard(move |message: &Message| match message.text() {
                Some(text) => guard.check(text),
                None => false,
            })
        }
    }

    impl<UpdateParser, ParserT, Err> MessageParser<UpdateParser, ParserT, Err> {
        pub fn or_with_id(self, guard: impl Guard<i32> + 'static) -> Self {
            self.or(move |message: &Message| guard.check(&message.id))
        }

        pub fn or_with_date(self, guard: impl Guard<i32> + 'static) -> Self {
            self.or(move |message: &Message| guard.check(&message.date))
        }

        pub fn or_with_chat(self, guard: impl Guard<types::Chat> + 'static) -> Self {
            self.or(move |message: &Message| guard.check(&message.chat))
        }

        pub fn or_with_chat_id(self, guard: impl Guard<i64> + 'static) -> Self {
            self.or(move |message: &Message| guard.check(&message.chat.id))
        }

        pub fn or_with_via_bot(self, guard: impl Guard<types::User> + 'static) -> Self {
            self.or(move |message: &Message| match &message.via_bot {
                Some(bot) => guard.check(bot),
                None => false,
            })
        }

        pub fn or_with_from(self, guard: impl Guard<types::User> + 'static) -> Self {
            self.or(move |message: &Message| match message.from() {
                Some(user) => guard.check(user),
                None => false,
            })
        }

        pub fn or_with_forward_from(
            self,
            guard: impl Guard<types::ForwardedFrom> + 'static,
        ) -> Self {
            self.or(move |message: &Message| match message.forward_from() {
                Some(user) => guard.check(user),
                None => false,
            })
        }

        pub fn or_with_forward_from_chat(self, guard: impl Guard<types::Chat> + 'static) -> Self {
            self.or(move |message: &Message| match message.forward_from_chat() {
                Some(chat) => guard.check(chat),
                None => false,
            })
        }

        pub fn or_with_forward_from_message_id(self, guard: impl Guard<i32> + 'static) -> Self {
            self.or(
                move |message: &Message| match message.forward_from_message_id() {
                    Some(chat) => guard.check(chat),
                    None => false,
                },
            )
        }

        pub fn or_with_forward_signature(self, guard: impl Guard<str> + 'static) -> Self {
            self.or(move |message: &Message| match message.forward_signature() {
                Some(chat) => guard.check(chat),
                None => false,
            })
        }

        pub fn or_with_forward_date(self, guard: impl Guard<i32> + 'static) -> Self {
            self.or(move |message: &Message| match message.forward_date() {
                Some(chat) => guard.check(chat),
                None => false,
            })
        }

        pub fn or_with_text(self, guard: impl Guard<str> + 'static) -> Self {
            self.or(move |message: &Message| match message.text() {
                Some(text) => guard.check(text),
                None => false,
            })
        }
    }

    pub struct MessageHandler<Parser, HandlerT, Err> {
        parser: Parser,
        handler: HandlerT,
        demux: Demux<Message, Err>,
        phantom: PhantomData<Err>,
    }

    impl<ParserT, Err, HandlerT> Handler<Update, Err, HandleFuture<Err>>
        for MessageHandler<ParserT, HandlerT, Err>
    where
        ParserT: Parser<Update, Message, (UpdateRest, ())>,
        HandlerT: Handler<Message, Err, HandleFuture<Err>>,
        Update: RecombineFrom<ParserT, From = Message, Rest = (UpdateRest, ())>,
    {
        fn handle(&self, update: Update) -> Result<HandleFuture<Err>, Update> {
            let ParserOut { data: mes, rest } = self.parser.parse(update)?;
            match self.demux.handle(mes) {
                Ok(fut) => Ok(fut),
                Err(upd) => self.handler.handle(upd).map_err(|e| {
                    <Update as RecombineFrom<ParserT>>::recombine(ParserOut::new(e, rest))
                }),
            }
        }
    }

    impl<ParserT, Err> UpdateParser<Update, Message, UpdateRest, Err, ParserT>
    where
        ParserT: Parser<Update, Message, UpdateRest>,
        Update: RecombineFrom<ParserT, From = Message, Rest = UpdateRest>,
    {
        pub fn common(self) -> MessageParser<ParserT, parser::Common, Err> {
            MessageParser::new(self.into_inner(), parser::Common)
        }

        pub fn new_chat_members(self) -> MessageParser<ParserT, parser::NewChatMembers, Err> {
            MessageParser::new(self.into_inner(), parser::NewChatMembers)
        }

        pub fn left_chat_member(self) -> MessageParser<ParserT, parser::LeftChatMember, Err> {
            MessageParser::new(self.into_inner(), parser::LeftChatMember)
        }

        pub fn new_chat_title(self) -> MessageParser<ParserT, parser::NewChatTitle, Err> {
            MessageParser::new(self.into_inner(), parser::NewChatTitle)
        }

        pub fn new_chat_photo(self) -> MessageParser<ParserT, parser::NewChatPhoto, Err> {
            MessageParser::new(self.into_inner(), parser::NewChatPhoto)
        }

        pub fn delete_chat_photo(self) -> MessageParser<ParserT, parser::DeleteChatPhoto, Err> {
            MessageParser::new(self.into_inner(), parser::DeleteChatPhoto)
        }

        pub fn group_chat_created(self) -> MessageParser<ParserT, parser::GroupChatCreated, Err> {
            MessageParser::new(self.into_inner(), parser::GroupChatCreated)
        }

        pub fn supergroup_chat_created(
            self,
        ) -> MessageParser<ParserT, parser::SupergroupChatCreated, Err> {
            MessageParser::new(self.into_inner(), parser::SupergroupChatCreated)
        }

        pub fn channel_chat_created(
            self,
        ) -> MessageParser<ParserT, parser::ChannelChatCreated, Err> {
            MessageParser::new(self.into_inner(), parser::ChannelChatCreated)
        }

        pub fn migrate(self) -> MessageParser<ParserT, parser::Migrate, Err> {
            MessageParser::new(self.into_inner(), parser::Migrate)
        }

        pub fn pinned(self) -> MessageParser<ParserT, parser::Pinned, Err> {
            MessageParser::new(self.into_inner(), parser::Pinned)
        }

        pub fn invoice(self) -> MessageParser<ParserT, parser::Invoice, Err> {
            MessageParser::new(self.into_inner(), parser::Invoice)
        }

        pub fn successful_payment(self) -> MessageParser<ParserT, parser::SuccessfulPayment, Err> {
            MessageParser::new(self.into_inner(), parser::SuccessfulPayment)
        }

        pub fn connected_website(self) -> MessageParser<ParserT, parser::ConnectedWebsite, Err> {
            MessageParser::new(self.into_inner(), parser::ConnectedWebsite)
        }

        pub fn passport_data(self) -> MessageParser<ParserT, parser::PassportData, Err> {
            MessageParser::new(self.into_inner(), parser::PassportData)
        }

        pub fn dice(self) -> MessageParser<ParserT, parser::Dice, Err> {
            MessageParser::new(self.into_inner(), parser::Dice)
        }
    }
}
