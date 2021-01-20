mod impls {
    use crate::core::{HandleResult, Handler, IntoHandler, MapParser, Parser, ParserOut, RecombineFrom, Guards, Guard, HandleFuture, Demux};
    use crate::handlers::parser::UpdateParser;
    use crate::updates::UpdateRest;
    use std::future::Future;
    use std::marker::PhantomData;
    use teloxide_core::types::{Message, Update};
    use teloxide_core::types;

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

    pub struct MessageParser<ParentParser, ParserT, Err> {
        parent: ParentParser,
        parser: ParserT,
        guards: Guards<Message>,
        phantom: PhantomData<Err>,
    }

    impl<ParentParser, ParserT, Err> MessageParser<ParentParser, ParserT, Err>
    where
        ParentParser: Parser<Update, Message, UpdateRest>,
        ParserT: Parser<Message, Message, ()> + 'static,
        Update: RecombineFrom<ParentParser, From = Message, Rest = UpdateRest>,
    {
        pub fn new(parent: ParentParser, parser: ParserT) -> Self {
            MessageParser {
                parent,
                parser,
                guards: Guards::new(),
                phantom: PhantomData,
            }
        }

        pub fn by<F, H, Fut>(
            self,
            f: F,
        ) -> MessageHandler<
            MapParser<ParentParser, ParserT, Message, UpdateRest, (), Message>,
            H,
            Err,
        >
        where
            H: Handler<Message, Err, Fut> + 'static,
            F: IntoHandler<H>,
            Fut: Future + Send + 'static,
            Fut::Output: Into<HandleResult<Err>>,
        {
            let MessageParser { parent, parser, guards, .. } = self;
            let parser = MapParser::new(parent, parser);
            MessageHandler {
                parser,
                handler: f.into_handler(),
                guards,
                phantom: PhantomData
            }
        }
    }

    impl<ParentParser, ParserT, Err> MessageParser<ParentParser, ParserT, Err> {
        pub fn with_guard(mut self, guard: impl Guard<Message> + 'static) -> Self {
            self.guards.add_guard(guard);
            self
        }

        pub fn with_id(mut self, guard: impl Guard<i32> + 'static) -> Self {
            self.with_guard(move |message: &Message| {
                guard.check(&message.id)
            })
        }

        pub fn with_date(mut self, guard: impl Guard<i32> + 'static) -> Self {
            self.with_guard(move |message: &Message| {
                guard.check(&message.date)
            })
        }

        pub fn with_chat(mut self, guard: impl Guard<types::Chat> + 'static) -> Self {
            self.with_guard(move |message: &Message| {
                guard.check(&message.chat)
            })
        }

        pub fn with_chat_id(mut self, guard: impl Guard<i64> + 'static) -> Self {
            self.with_guard(move |message: &Message| {
                guard.check(&message.chat.id)
            })
        }

        pub fn with_via_bot(mut self, guard: impl Guard<types::User> + 'static) -> Self {
            self.with_guard(move |message: &Message| {
                match &message.via_bot {
                    Some(bot) => guard.check(bot),
                    None => false,
                }
            })
        }

        pub fn with_from(mut self, guard: impl Guard<types::User> + 'static) -> Self {
            self.with_guard(move |message: &Message| {
                match message.from() {
                    Some(user) => guard.check(user),
                    None => false,
                }
            })
        }

        pub fn with_forward_from(mut self, guard: impl Guard<types::ForwardedFrom> + 'static) -> Self {
            self.with_guard(move |message: &Message| {
                match message.forward_from() {
                    Some(user) => guard.check(user),
                    None => false,
                }
            })
        }

        pub fn with_forward_from_chat(mut self, guard: impl Guard<types::Chat> + 'static) -> Self {
            self.with_guard(move |message: &Message| {
                match message.forward_from_chat() {
                    Some(chat) => guard.check(chat),
                    None => false,
                }
            })
        }

        pub fn with_forward_from_message_id(mut self, guard: impl Guard<i32> + 'static) -> Self {
            self.with_guard(move |message: &Message| {
                match message.forward_from_message_id() {
                    Some(chat) => guard.check(chat),
                    None => false,
                }
            })
        }

        pub fn with_forward_signature(mut self, guard: impl Guard<str> + 'static) -> Self {
            self.with_guard(move |message: &Message| {
                match message.forward_signature() {
                    Some(chat) => guard.check(chat),
                    None => false,
                }
            })
        }

        pub fn with_forward_date(mut self, guard: impl Guard<i32> + 'static) -> Self {
            self.with_guard(move |message: &Message| {
                match message.forward_date() {
                    Some(chat) => guard.check(chat),
                    None => false,
                }
            })
        }
    }

    pub struct MessageHandler<Parser, HandlerT, Err> {
        parser: Parser,
        handler: HandlerT,
        guards: Guards<Message>,
        phantom: PhantomData<Err>,
    }

    impl<ParserT, Err, HandlerT> Handler<Update, Err, HandleFuture<Err>> for MessageHandler<ParserT, HandlerT, Err>
    where
        ParserT: Parser<Update, Message, (UpdateRest, ())>,
        HandlerT: Handler<Message, Err, HandleFuture<Err>>,
        Update: RecombineFrom<ParserT, From = Message, Rest = (UpdateRest, ())>,
    {
        fn handle(&self, update: Update) -> Result<HandleFuture<Err>, Update> {
            let ParserOut { data: mes, rest } = self.parser.parse(update)?;
            if self.guards.check(&mes) {
                match self.handler.handle(mes) {
                    Ok(fut) => Ok(fut),
                    Err(mes) => Err(<Update as RecombineFrom<ParserT>>::recombine(ParserOut::new(mes, rest))),
                }
            }
            else {
                Err(<Update as RecombineFrom<ParserT>>::recombine(ParserOut::new(mes, rest)))
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
