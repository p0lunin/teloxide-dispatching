mod impls {
    use crate::core::{
        HandleResult, Handler, IntoHandler, MapParser, Parser, ParserHandler, ParserOut,
        RecombineFrom,
    };
    use crate::handlers::parser::UpdateParser;
    use crate::updates::UpdateRest;
    use std::future::Future;
    use std::marker::PhantomData;
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

    pub struct MessageParser<ParentParser, ParserT, Err> {
        parent: ParentParser,
        parser: ParserT,
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
                phantom: PhantomData,
            }
        }

        pub fn by<F, H, Fut>(
            self,
            f: F,
        ) -> ParserHandler<
            MapParser<ParentParser, ParserT, Message, UpdateRest, (), Message>,
            Update,
            Message,
            (UpdateRest, ()),
            Err,
            H,
            Fut,
        >
        where
            H: Handler<Message, Err, Fut> + 'static,
            F: IntoHandler<H>,
            Fut: Future + Send + 'static,
            Fut::Output: Into<HandleResult<Err>>,
        {
            let MessageParser { parent, parser, .. } = self;
            let parser = MapParser::new(parent, parser);
            ParserHandler::new(parser, f)
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
